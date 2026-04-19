use gtk4::prelude::*;
use glib::prelude::*;
use std::time::Duration;
use super::helpers::*;

pub fn new_wallpaper(cfg: &crate::config::Config) -> gtk4::Widget {
    let btn = gtk4::Button::with_label("");
    btn.set_has_frame(false);
    btn.set_widget_name("custom-wallpaper");
    btn.set_tooltip_text(Some("Shuffle wallpaper"));

    let wp_dir = cfg.wallpaper.dir.clone();
    let auto_switch = cfg.wallpaper.auto_switch;
    let interval_min = cfg.wallpaper.interval.max(1) as u64;
    let on_click = cfg.wallpaper.on_click.clone();

    let (popup_wp, menu) = make_popup();
    menu.set_widget_name("wallpaper-menu");

    let title = gtk4::Label::new(Some("Wallpaper"));
    title.set_xalign(0.0);
    menu.append(&title);

    let shuffle_btn = gtk4::Button::with_label("Shuffle");
    shuffle_btn.set_has_frame(false);
    let wp_dir2 = wp_dir.clone();
    shuffle_btn.connect_clicked(move |_| {
        set_random_wallpaper(&wp_dir2);
    });
    menu.append(&shuffle_btn);

    attach_hover_popup(&btn, &popup_wp, || {});

    // Click
    let wp_dir3 = wp_dir.clone();
    let on_click2 = on_click.clone();
    attach_click(&btn,
        move || {
            if !on_click2.is_empty() {
                run_detached(&on_click2);
            } else {
                set_random_wallpaper(&wp_dir3);
            }
        },
        || {}
    );

    // Auto-switch timer
    if auto_switch {
        let wp_dir4 = wp_dir.clone();
        let btn_weak = btn.downgrade();
        glib::timeout_add_local(Duration::from_secs(interval_min * 60), move || {
            if btn_weak.upgrade().is_none() { return glib::ControlFlow::Break; }
            set_random_wallpaper(&wp_dir4);
            glib::ControlFlow::Continue
        });
    }

    btn.upcast()
}

fn list_wallpapers(dir: &str) -> Vec<std::path::PathBuf> {
    let exts = ["jpg", "jpeg", "png", "webp", "gif"];
    let Ok(entries) = std::fs::read_dir(dir) else { return vec![] };
    entries
        .flatten()
        .filter(|e| {
            let path = e.path();
            if let Some(ext) = path.extension() {
                exts.contains(&ext.to_string_lossy().to_lowercase().as_str())
            } else {
                false
            }
        })
        .map(|e| e.path())
        .collect()
}

fn set_random_wallpaper(dir: &str) {
    let files = list_wallpapers(dir);
    if files.is_empty() { return; }
    let idx = (std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos() as usize) % files.len();
    let path = &files[idx];
    let path_str = path.to_string_lossy().to_string();
    let _ = std::process::Command::new("hyprctl")
        .args(&["hyprpaper", "preload", &path_str])
        .spawn();
    let wall_cmd = format!(", {}", path_str);
    let _ = std::process::Command::new("hyprctl")
        .args(&["hyprpaper", "wallpaper", &wall_cmd])
        .spawn();
}
