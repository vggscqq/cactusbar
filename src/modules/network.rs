use gtk4::prelude::*;
use glib::prelude::*;
use std::time::Duration;
use super::helpers::*;

pub fn new_network(cfg: &crate::config::Config) -> gtk4::Box {
    let module = TextModule::new("network");
    remove_children(&module.container);
    module.container.set_visible(true);

    let icon_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 4);
    let net_icon = gtk4::Image::from_icon_name("network-wireless-signal-excellent-symbolic");
    icon_box.append(&net_icon);

    let val_label = gtk4::Label::new(None);
    val_label.set_xalign(0.0);
    if cfg.network.show_text {
        icon_box.append(&val_label);
    }
    module.container.append(&icon_box);

    let (popup_net, menu) = make_popup();
    menu.set_widget_name("wifi-menu");

    let wifi_title = gtk4::Label::new(Some("Wi-Fi"));
    wifi_title.set_widget_name("wifi-menu-title");
    wifi_title.set_xalign(0.0);
    menu.append(&wifi_title);

    let list_box = gtk4::Box::new(gtk4::Orientation::Vertical, 2);
    menu.append(&list_box);

    let on_click = cfg.network.on_click.clone();

    {
        let list_weak = list_box.downgrade();
        attach_hover_popup(&module.container, &popup_net, move || {
            if let Some(list) = list_weak.upgrade() { refresh_wifi_list(&list); }
        });
    }

    // Right click -> network manager
    attach_click(&module.container,
        || {},
        move || { run_detached(&on_click); }
    );

    let icon_weak = net_icon.downgrade();
    let label_weak = val_label.downgrade();
    let container_weak = module.container.downgrade();
    let show_text = cfg.network.show_text;

    glib::timeout_add_local(Duration::from_secs(5), move || {
        if container_weak.upgrade().is_none() {
            return glib::ControlFlow::Break;
        }
        let (icon_name, ssid) = read_network_state();
        if let Some(i) = icon_weak.upgrade() {
            i.set_icon_name(Some(&icon_name));
        }
        if show_text {
            if let Some(l) = label_weak.upgrade() {
                l.set_label(&ssid);
            }
        }
        glib::ControlFlow::Continue
    });

    module.container
}

fn read_network_state() -> (String, String) {
    // Check active wifi
    let out = run_cmd(&["nmcli", "-t", "-f", "ACTIVE,SSID,SIGNAL", "dev", "wifi"]);
    for line in out.lines() {
        let parts: Vec<&str> = line.splitn(3, ':').collect();
        if parts.len() >= 3 && parts[0] == "yes" {
            let ssid = parts[1].to_string();
            let signal = parts[2].parse::<i32>().unwrap_or(0);
            let icon = if signal >= 80 {
                "network-wireless-signal-excellent-symbolic"
            } else if signal >= 60 {
                "network-wireless-signal-good-symbolic"
            } else if signal >= 40 {
                "network-wireless-signal-ok-symbolic"
            } else {
                "network-wireless-signal-weak-symbolic"
            };
            return (icon.to_string(), ssid);
        }
    }
    // Check ethernet
    let eth = run_cmd(&["nmcli", "-t", "-f", "STATE", "general"]);
    if eth.trim().contains("connected") {
        return ("network-wired-symbolic".to_string(), String::new());
    }
    ("network-offline-symbolic".to_string(), String::new())
}

fn refresh_wifi_list(list: &gtk4::Box) {
    remove_children(list);
    let out = run_cmd(&["nmcli", "-t", "-f", "ACTIVE,SSID,SIGNAL,SECURITY", "dev", "wifi"]);
    let mut networks: Vec<(bool, String, i32, String)> = Vec::new();
    for line in out.lines() {
        let parts: Vec<&str> = line.splitn(4, ':').collect();
        if parts.len() < 4 { continue; }
        let active = parts[0] == "yes";
        let ssid = parts[1].to_string();
        if ssid.is_empty() { continue; }
        let signal = parts[2].parse::<i32>().unwrap_or(0);
        let security = parts[3].to_string();
        networks.push((active, ssid, signal, security));
    }
    networks.sort_by(|a, b| b.2.cmp(&a.2));

    if networks.is_empty() {
        let lbl = gtk4::Label::new(Some("No networks found"));
        lbl.set_xalign(0.0);
        list.append(&lbl);
        return;
    }

    for (active, ssid, signal, _security) in networks.iter().take(10) {
        let label_text = format!("{} {}%{}", ssid, signal, if *active { " ✓" } else { "" });
        let row = gtk4::Button::with_label(&label_text);
        row.set_has_frame(false);
        row.add_css_class("wifi-network-row");
        if *active { row.add_css_class("active"); }
        let ssid2 = ssid.clone();
        row.connect_clicked(move |_| {
            run_detached_args("nmcli", &["dev", "wifi", "connect", &ssid2]);
        });
        list.append(&row);
    }
}
