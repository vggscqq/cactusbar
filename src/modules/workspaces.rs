use gtk4::prelude::*;
use glib::prelude::*;
use std::time::Duration;
use super::helpers::*;

#[derive(Debug, Clone)]
struct WorkspaceData {
    active_ws_id: i64,
    ws_ids: Vec<i64>,
    ws_clients: std::collections::HashMap<i64, Vec<String>>,
}

fn fetch_workspace_data(monitor_name: &str) -> WorkspaceData {
    let clients_str = run_cmd(&["hyprctl", "-j", "clients"]);
    let ws_str = run_cmd(&["hyprctl", "-j", "workspaces"]);
    let mons_str = run_cmd(&["hyprctl", "-j", "monitors"]);

    let clients: Vec<serde_json::Value> = serde_json::from_str(&clients_str).unwrap_or_default();
    let workspaces: Vec<serde_json::Value> = serde_json::from_str(&ws_str).unwrap_or_default();
    let monitors: Vec<serde_json::Value> = serde_json::from_str(&mons_str).unwrap_or_default();

    let active_ws_id: i64 = monitors.iter()
        .find(|m| monitor_name.is_empty() || m["name"].as_str().unwrap_or("") == monitor_name)
        .and_then(|m| m["activeWorkspace"]["id"].as_i64())
        .unwrap_or(0);

    let mut monitor_ws_ids: std::collections::HashSet<i64> = workspaces.iter()
        .filter(|w| monitor_name.is_empty() || w["monitor"].as_str().unwrap_or("") == monitor_name)
        .filter_map(|w| w["id"].as_i64())
        .collect();
    if active_ws_id > 0 { monitor_ws_ids.insert(active_ws_id); }

    let mut ws_clients: std::collections::HashMap<i64, Vec<String>> = std::collections::HashMap::new();
    for ws_id in &monitor_ws_ids { ws_clients.insert(*ws_id, vec![]); }
    for client in &clients {
        let ws_id = client["workspace"]["id"].as_i64().unwrap_or(0);
        if ws_id > 0 && monitor_ws_ids.contains(&ws_id) {
            let class = client["class"].as_str().unwrap_or("").to_lowercase();
            ws_clients.entry(ws_id).or_default().push(class);
        }
    }

    let mut ws_ids: Vec<i64> = monitor_ws_ids.into_iter().collect();
    ws_ids.sort();
    WorkspaceData { active_ws_id, ws_ids, ws_clients }
}

fn apply_workspace_data(box_: &gtk4::Box, data: WorkspaceData) {
    remove_children(box_);
    for id in data.ws_ids {
        let btn = gtk4::Button::new();
        btn.set_has_frame(false);
        if id == data.active_ws_id { btn.add_css_class("active"); }

        let content = gtk4::Box::new(gtk4::Orientation::Horizontal, 2);
        content.append(&gtk4::Label::new(Some(&id.to_string())));

        if let Some(classes) = data.ws_clients.get(&id) {
            for class in classes.iter().take(5) {
                let img = gtk4::Image::from_icon_name(class);
                img.set_pixel_size(12);
                content.append(&img);
            }
        }
        btn.set_child(Some(&content));
        btn.connect_clicked(move |_| {
            run_detached_args("hyprctl", &["dispatch", "workspace", &id.to_string()]);
        });
        box_.append(&btn);
    }
}

pub fn new_workspaces(monitor_name: &str) -> gtk4::Box {
    let box_ = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
    box_.set_widget_name("workspaces");

    let monitor_name = monitor_name.to_string();
    let (tx, rx) = async_channel::bounded::<WorkspaceData>(1);

    let refresh = {
        let monitor_name = monitor_name.clone();
        let tx = tx.clone();
        move || {
            let monitor_name = monitor_name.clone();
            let tx = tx.clone();
            std::thread::spawn(move || {
                let data = fetch_workspace_data(&monitor_name);
                tx.send_blocking(data).ok();
            });
        }
    };

    // Main thread receiver
    let box_weak = box_.downgrade();
    glib::MainContext::default().spawn_local(async move {
        while let Ok(data) = rx.recv().await {
            let box_ = match box_weak.upgrade() { Some(b) => b, None => break };
            apply_workspace_data(&box_, data);
        }
    });

    refresh();

    // Poll fallback
    let refresh_poll = refresh.clone();
    let tx_poll = tx.clone();
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
            if event.starts_with("workspace>>") || event.starts_with("activewindow>>")
                || event.starts_with("openwindow>>") || event.starts_with("closewindow>>")
                || event.starts_with("movewindow>>") || event.starts_with("destroyworkspace>>")
                || event.starts_with("createworkspace>>")
            {
                refresh();
            }
        }
    });

    box_
}
