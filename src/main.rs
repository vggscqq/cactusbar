mod app;
mod config;
mod layershell;
mod modules;
mod services;

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use gtk4::prelude::*;
use gio::prelude::*;
use glib::prelude::*;

pub static RUNTIME: OnceLock<tokio::runtime::Runtime> = OnceLock::new();

thread_local! {
    pub static APP: std::cell::RefCell<Option<gtk4::Application>> = std::cell::RefCell::new(None);
}

fn configure_graphics_backend() {
    if std::env::var_os("GSK_RENDERER").is_none() {
        // Vulkan can cause popup flicker on some Intel Mesa stacks.
        std::env::set_var("GSK_RENDERER", "ngl");
        log::info!("GSK_RENDERER not set; defaulting to ngl");
    }
}

fn main() {
    env_logger::init();
    configure_graphics_backend();
    RUNTIME.get_or_init(|| tokio::runtime::Runtime::new().expect("tokio runtime"));

    let args: Vec<String> = std::env::args().collect();
    let mut css_path = String::new();
    let mut gtk_args: Vec<String> = vec![args[0].clone()];
    let mut i = 1;
    while i < args.len() {
        if args[i] == "--css" && i + 1 < args.len() {
            css_path = args[i + 1].clone();
            i += 2;
        } else {
            gtk_args.push(args[i].clone());
            i += 1;
        }
    }

    let application = gtk4::Application::new(
        Some("dev.fikus.statusbar"),
        gio::ApplicationFlags::empty(),
    );

    let css_path = std::rc::Rc::new(css_path);
    application.connect_activate(move |app| {
        crate::APP.with(|a| *a.borrow_mut() = Some(app.clone()));
        let css_path = css_path.clone();
        let display = gdk4::Display::default();

        if display.is_none() {
            let window = app::new(app, &css_path, None, "");
            window.present();
            return;
        }
        let display = display.unwrap();

        let windows: std::rc::Rc<Mutex<HashMap<String, gtk4::ApplicationWindow>>> =
            std::rc::Rc::new(Mutex::new(HashMap::new()));

        let windows_spawn = windows.clone();
        let app_spawn = app.clone();
        let css_spawn = css_path.clone();
        let monitors = display.monitors();

        for i in 0..monitors.n_items() {
            if let Some(obj) = monitors.item(i) {
                if let Ok(mon) = obj.downcast::<gdk4::Monitor>() {
                    let name = mon.connector().unwrap_or_default().to_string();
                    if !windows_spawn.lock().unwrap().contains_key(&name) {
                        let window = app::new(&app_spawn, &css_spawn, Some(&mon), &name);
                        windows_spawn.lock().unwrap().insert(name, window.clone());
                        window.present();
                    }
                }
            }
        }

        let windows_cb = windows.clone();
        let app_cb = app.clone();
        let css_cb = css_path.clone();
        monitors.connect_items_changed(move |monitors, _pos, _removed, _added| {
            let mut current: HashMap<String, gdk4::Monitor> = HashMap::new();
            for i in 0..monitors.n_items() {
                if let Some(obj) = monitors.item(i) {
                    if let Ok(mon) = obj.downcast::<gdk4::Monitor>() {
                        let name = mon.connector().unwrap_or_default().to_string();
                        current.insert(name, mon);
                    }
                }
            }
            let gone: Vec<String> = {
                let lock = windows_cb.lock().unwrap();
                lock.keys().filter(|k| !current.contains_key(*k)).cloned().collect()
            };
            for name in gone {
                if let Some(w) = windows_cb.lock().unwrap().remove(&name) {
                    w.destroy();
                }
            }
            for (name, mon) in &current {
                if !windows_cb.lock().unwrap().contains_key(name) {
                    let window = app::new(&app_cb, &css_cb, Some(mon), name);
                    windows_cb.lock().unwrap().insert(name.clone(), window.clone());
                    window.present();
                }
            }
        });
    });

    let exit_code = application.run_with_args(&gtk_args);
    std::process::exit(exit_code.into());
}
