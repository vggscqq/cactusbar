use gtk4::prelude::*;
use glib::prelude::*;
use std::time::Duration;
use super::helpers::*;

#[derive(Debug, Clone)]
struct FocusedAppData {
    ws_id: i64,
    title: String,
    class: String,
}

fn fetch_focused_app(monitor_name: &str) -> FocusedAppData {
    let active_str = run_cmd(&["hyprctl", "-j", "activewindow"]);
    let mons_str = run_cmd(&["hyprctl", "-j", "monitors"]);
    let active: serde_json::Value = serde_json::from_str(&active_str).unwrap_or_default();
    let monitors: Vec<serde_json::Value> = serde_json::from_str(&mons_str).unwrap_or_default();

    let ws_id: i64 = monitors.iter()
        .find(|m| monitor_name.is_empty() || m["name"].as_str().unwrap_or("") == monitor_name)
        .and_then(|m| m["activeWorkspace"]["id"].as_i64())
        .unwrap_or(0);

    FocusedAppData {
        ws_id,
        title: active["title"].as_str().unwrap_or("").to_string(),
        class: active["class"].as_str().unwrap_or("").to_string(),
    }
}

pub fn new_focused_app(
    cfg: &crate::config::Config,
    _win: &gtk4::ApplicationWindow,
    monitor_name: &str,
) -> gtk4::Box {
    let show_empty = cfg.focused_app_config.show_empty_workspace;
    let empty_text = cfg.focused_app_config.empty_text.clone();

    let box_ = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
    box_.set_widget_name("focused-app");
    box_.set_visible(false);

    let badge_shell = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
    badge_shell.set_widget_name("focused-app-badge");
    let ws_num_label = gtk4::Label::new(None);
    ws_num_label.add_css_class("focused-app-ws-badge");
    badge_shell.append(&ws_num_label);

    let text_shell = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
    text_shell.set_widget_name("focused-app-text");
    let app_icon = gtk4::Image::new();
    app_icon.set_pixel_size(14);
    let title_label = gtk4::Label::new(None);
    title_label.set_single_line_mode(true);
    title_label.set_ellipsize(pango::EllipsizeMode::End);
    title_label.set_max_width_chars(32);
    title_label.set_xalign(0.0);
    text_shell.append(&app_icon);
    text_shell.append(&title_label);

    box_.append(&badge_shell);
    box_.append(&text_shell);

    let monitor_name = monitor_name.to_string();
    let (tx, rx) = async_channel::bounded::<FocusedAppData>(1);

    let refresh = {
        let monitor_name = monitor_name.clone();
        let tx = tx.clone();
        move || {
            let monitor_name = monitor_name.clone();
            let tx = tx.clone();
            std::thread::spawn(move || {
                let data = fetch_focused_app(&monitor_name);
                tx.send_blocking(data).ok();
            });
        }
    };

    // Main thread receiver
    let box_weak = box_.downgrade();
    let ws_weak = ws_num_label.downgrade();
    let icon_weak = app_icon.downgrade();
    let title_weak = title_label.downgrade();
    glib::MainContext::default().spawn_local(async move {
        while let Ok(data) = rx.recv().await {
            let box_ = match box_weak.upgrade() { Some(b) => b, None => break };
            let ws_label = match ws_weak.upgrade() { Some(l) => l, None => break };
            let app_icon = match icon_weak.upgrade() { Some(i) => i, None => break };
            let title_label = match title_weak.upgrade() { Some(l) => l, None => break };

            if data.title.is_empty() && data.class.is_empty() {
                if show_empty && data.ws_id > 0 {
                    ws_label.set_label(&data.ws_id.to_string());
                    app_icon.set_icon_name(Some("application-x-executable"));
                    title_label.set_label(&empty_text);
                    box_.set_visible(true);
                } else {
                    box_.set_visible(false);
                }
                continue;
            }

            ws_label.set_label(&data.ws_id.to_string());
            app_icon.set_icon_name(Some(&data.class.to_lowercase()));
            title_label.set_label(&truncate(&data.title, 40));
            box_.set_visible(true);
        }
    });

    refresh();

    // Poll fallback
    let refresh_poll = refresh.clone();
    let box_weak2 = box_.downgrade();
    glib::timeout_add_local(Duration::from_secs(1), move || {
        if box_weak2.upgrade().is_none() { return glib::ControlFlow::Break; }
        refresh_poll();
        glib::ControlFlow::Continue
    });

    // Hyprland events
    let rx_ev = super::events::hypr_event_channel();
    glib::MainContext::default().spawn_local(async move {
        while let Ok(event) = rx_ev.recv().await {
            if event.starts_with("activewindow>>") || event.starts_with("openwindow>>")
                || event.starts_with("closewindow>>") || event.starts_with("workspace>>")
            {
                refresh();
            }
        }
    });

    box_
}
