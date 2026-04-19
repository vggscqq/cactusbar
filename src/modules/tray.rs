use gtk4::prelude::*;
use glib::prelude::*;
use super::helpers::*;

pub fn new_tray() -> gtk4::Box {
    let box_ = gtk4::Box::new(gtk4::Orientation::Horizontal, 2);
    box_.set_widget_name("tray");

    // Start SNI watcher
    let (sni_tx, sni_rx) = async_channel::bounded::<crate::services::sni::SniEvent>(32);
    crate::services::sni::start_watcher(sni_tx);

    let box_weak = box_.downgrade();
    glib::MainContext::default().spawn_local(async move {
        while let Ok(event) = sni_rx.recv().await {
            let box_ = match box_weak.upgrade() {
                Some(b) => b,
                None => break,
            };
            match event {
                crate::services::sni::SniEvent::ItemRegistered(id) => {
                    add_tray_item(&box_, &id);
                }
                crate::services::sni::SniEvent::ItemUnregistered(id) => {
                    remove_tray_item(&box_, &id);
                }
            }
        }
    });

    box_
}

fn add_tray_item(box_: &gtk4::Box, id: &str) {
    // Avoid duplicate
    let mut child = box_.first_child();
    while let Some(c) = child {
        if c.widget_name() == id { return; }
        child = c.next_sibling();
    }

    let btn = gtk4::Button::new();
    btn.set_has_frame(false);
    btn.set_widget_name(id);
    btn.add_css_class("tray-item");

    let (bus, path) = parse_sni_id(id);
    let icon_name = get_sni_icon_name(&bus, &path);

    let img = if !icon_name.is_empty() {
        gtk4::Image::from_icon_name(&icon_name)
    } else {
        gtk4::Image::from_icon_name("application-x-executable")
    };
    img.set_pixel_size(16);
    btn.set_child(Some(&img));

    let bus2 = bus.clone();
    let path2 = path.clone();
    btn.connect_clicked(move |_| {
        activate_sni_item(&bus2, &path2);
    });

    box_.append(&btn);
}

fn remove_tray_item(box_: &gtk4::Box, id: &str) {
    let mut child = box_.first_child();
    while let Some(c) = child {
        if c.widget_name() == id { box_.remove(&c); return; }
        child = c.next_sibling();
    }
}

fn parse_sni_id(id: &str) -> (String, String) {
    if let Some(idx) = id.find('/') {
        (id[..idx].to_string(), id[idx..].to_string())
    } else {
        (id.to_string(), "/StatusNotifierItem".to_string())
    }
}

fn get_sni_icon_name(bus: &str, path: &str) -> String {
    let out = run_cmd(&[
        "gdbus", "call", "--session",
        "--dest", bus, "--object-path", path,
        "--method", "org.freedesktop.DBus.Properties.Get",
        "org.kde.StatusNotifierItem", "IconName",
    ]);
    if let Some(start) = out.find('\'') {
        if let Some(end) = out.rfind('\'') {
            if end > start { return out[start+1..end].to_string(); }
        }
    }
    String::new()
}

fn activate_sni_item(bus: &str, path: &str) {
    let _ = std::process::Command::new("gdbus")
        .args(&["call", "--session", "--dest", bus,
                "--object-path", path,
                "--method", "org.kde.StatusNotifierItem.Activate", "0", "0"])
        .spawn();
}
