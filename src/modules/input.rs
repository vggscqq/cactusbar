use gtk4::prelude::*;
use glib::prelude::*;
use std::time::Duration;
use super::helpers::*;

pub fn new_keyboard_state() -> gtk4::Box {
    let module = TextModule::new("keyboard-state");
    let container_weak = module.container.downgrade();
    let label_weak = module.label.downgrade();

    glib::timeout_add_local(Duration::from_secs(2), move || {
        if container_weak.upgrade().is_none() { return glib::ControlFlow::Break; }
        let num = lock_state("numlock");
        let caps = lock_state("capslock");
        let text = format!("NUM {} CAPS {}", lock_icon(num), lock_icon(caps));
        if let Some(l) = label_weak.upgrade() { l.set_label(&text); }
        if let Some(c) = container_weak.upgrade() { c.set_visible(true); }
        glib::ControlFlow::Continue
    });

    module.container
}

fn lock_state(name: &str) -> bool {
    let dir = match std::fs::read_dir("/sys/class/leds") {
        Ok(d) => d,
        Err(_) => return false,
    };
    for entry in dir.flatten() {
        let entry_name = entry.file_name();
        let entry_name = entry_name.to_string_lossy();
        if entry_name.contains(name) {
            let brightness_path = entry.path().join("brightness");
            if let Ok(data) = std::fs::read_to_string(&brightness_path) {
                return data.trim() == "1";
            }
        }
    }
    false
}

fn lock_icon(locked: bool) -> &'static str {
    if locked { "" } else { "" }
}

pub fn new_language(cfg: &crate::config::Config) -> gtk4::Box {
    let module = TextModule::new("language");

    let popover = gtk4::Popover::new();
    popover.add_css_class("status-popup");
    popover.set_has_arrow(false);
    popover.set_autohide(true);
    popover.set_parent(&module.container);

    let menu = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
    menu.set_widget_name("language-menu");
    popover.set_child(Some(&menu));

    let languages = cfg.languages.clone();
    let current_layout = std::rc::Rc::new(std::cell::RefCell::new(String::new()));

    let motion = gtk4::EventControllerMotion::new();
    let popover_weak = popover.downgrade();
    motion.connect_enter(move |_, _, _| {
        if let Some(p) = popover_weak.upgrade() { p.popup(); }
    });
    let popover_weak2 = popover.downgrade();
    motion.connect_leave(move |_| {
        if let Some(p) = popover_weak2.upgrade() {
            glib::timeout_add_local_once(Duration::from_millis(100), move || { p.popdown(); });
        }
    });
    module.container.add_controller(motion);

    let refresh_popup = {
        let menu_weak = menu.downgrade();
        let languages = languages.clone();
        let current_layout = current_layout.clone();
        move || {
            let menu = match menu_weak.upgrade() { Some(m) => m, None => return };
            remove_children(&menu);
            let layout = current_layout.borrow().clone();
            for (i, entry) in languages.iter().enumerate() {
                let active = layout.to_lowercase().contains(&entry.r#match.to_lowercase());
                let lbl_text = if active { format!("· {}", entry.label) } else { entry.label.clone() };
                let row = gtk4::Button::with_label(&lbl_text);
                row.set_has_frame(false);
                row.add_css_class("language-row");
                if active { row.add_css_class("active"); }
                let idx = i.to_string();
                row.connect_clicked(move |_| {
                    run_detached_args("hyprctl", &["switchxkblayout", "at-translated-set-2-keyboard", &idx]);
                });
                menu.append(&row);
            }
        }
    };

    let fmt_layout = {
        let languages = languages.clone();
        move |layout: &str| -> String {
            if !languages.is_empty() {
                let lower = layout.to_lowercase();
                for entry in &languages {
                    if lower.contains(&entry.r#match.to_lowercase()) {
                        return entry.label.clone();
                    }
                }
                return layout.chars().take(3).collect::<String>().to_uppercase();
            }
            format_language(layout)
        }
    };

    let container_weak = module.container.downgrade();
    let label_weak = module.label.downgrade();
    let current_layout2 = current_layout.clone();
    let fmt_layout2 = fmt_layout.clone();
    let refresh_popup2 = refresh_popup.clone();

    // Initial state from hyprctl
    let out = run_cmd(&["hyprctl", "-j", "devices"]);
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&out) {
        if let Some(keyboards) = json["keyboards"].as_array() {
            if let Some(kb) = keyboards.iter().find(|k| {
                k["name"].as_str().unwrap_or("").contains("at-translated")
            }) {
                let layout = kb["active_keymap"].as_str().unwrap_or("").to_string();
                *current_layout.borrow_mut() = layout.clone();
                let text = fmt_layout(&layout);
                module.label.set_label(&text);
                module.container.set_visible(!text.is_empty());
            }
        }
    }

    // Subscribe to hypr events
    let rx = super::events::hypr_event_channel();
    glib::MainContext::default().spawn_local(async move {
        while let Ok(event) = rx.recv().await {
            if let Some(rest) = event.strip_prefix("activelayout>>") {
                if let Some((_kbd, layout)) = rest.split_once(',') {
                    let layout = layout.to_string();
                    *current_layout2.borrow_mut() = layout.clone();
                    let text = fmt_layout2(&layout);
                    if let Some(l) = label_weak.upgrade() { l.set_label(&text); }
                    if let Some(c) = container_weak.upgrade() { c.set_visible(!text.is_empty()); }
                    refresh_popup2();
                }
            }
        }
    });

    module.container
}

fn format_language(layout: &str) -> String {
    let trimmed = layout.trim();
    if trimmed.is_empty() { return String::new(); }
    let first_word = trimmed.split_whitespace().next().unwrap_or(trimmed);
    first_word.chars().take(3).collect::<String>().to_uppercase()
}

pub fn new_mode() -> gtk4::Box {
    let module = TextModule::new("submap");
    let container_weak = module.container.downgrade();
    let label_weak = module.label.downgrade();

    glib::timeout_add_local(Duration::from_secs(2), move || {
        if container_weak.upgrade().is_none() { return glib::ControlFlow::Break; }
        let mode = run_cmd(&["hyprctl", "submap"]);
        let mode = mode.trim().to_string();
        let visible = !mode.is_empty() && mode != "default";
        if let Some(l) = label_weak.upgrade() { l.set_label(&mode); }
        if let Some(c) = container_weak.upgrade() { c.set_visible(visible); }
        glib::ControlFlow::Continue
    });

    let container_weak2 = module.container.downgrade();
    let label_weak2 = module.label.downgrade();
    let rx = super::events::hypr_event_channel();
    glib::MainContext::default().spawn_local(async move {
        while let Ok(event) = rx.recv().await {
            if let Some(mode) = event.strip_prefix("submap>>") {
                let mode = mode.to_string();
                let visible = !mode.is_empty() && mode != "default";
                if let Some(l) = label_weak2.upgrade() { l.set_label(&mode); }
                if let Some(c) = container_weak2.upgrade() { c.set_visible(visible); }
            }
        }
    });

    module.container
}

pub fn new_scratchpad() -> gtk4::Box {
    let module = TextModule::new("scratchpad");
    let container_weak = module.container.downgrade();
    let label_weak = module.label.downgrade();

    glib::timeout_add_local(Duration::from_secs(2), move || {
        if container_weak.upgrade().is_none() { return glib::ControlFlow::Break; }
        let out = run_cmd(&["hyprctl", "-j", "clients"]);
        let clients: Vec<serde_json::Value> = serde_json::from_str(&out).unwrap_or_default();
        let count = clients.iter().filter(|c| {
            c["workspace"]["id"].as_i64().map(|id| id < 0).unwrap_or(false)
        }).count();
        let text = if count > 0 { format!("  {}", count) } else { String::new() };
        if let Some(l) = label_weak.upgrade() { l.set_label(&text); }
        if let Some(c) = container_weak.upgrade() { c.set_visible(!text.is_empty()); }
        glib::ControlFlow::Continue
    });

    module.container
}
