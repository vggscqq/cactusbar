use gtk4::prelude::*;
use glib::prelude::*;
use std::time::Duration;
use super::helpers::*;

pub fn new_bluetooth(_cfg: &crate::config::Config) -> gtk4::Box {
    let module = TextModule::new("bluetooth");
    remove_children(&module.container);
    module.container.set_visible(true);

    let icon = gtk4::Image::from_icon_name("bluetooth-symbolic");
    icon.set_pixel_size(16);
    module.container.append(&icon);

    let (popup, menu) = make_popup();
    menu.set_widget_name("bluetooth-menu");

    let title = gtk4::Label::new(Some("Bluetooth"));
    title.set_widget_name("bluetooth-menu-title");
    title.set_xalign(0.0);
    menu.append(&title);

    let device_list = gtk4::Box::new(gtk4::Orientation::Vertical, 2);
    menu.append(&device_list);

    {
        let list_weak = device_list.downgrade();
        attach_hover_popup(&module.container, &popup, move || {
            if let Some(list) = list_weak.upgrade() { refresh_bt_list(&list); }
        });
    }

    let container_weak = module.container.downgrade();
    let icon_weak = icon.downgrade();

    glib::timeout_add_local(Duration::from_secs(5), move || {
        if container_weak.upgrade().is_none() {
            return glib::ControlFlow::Break;
        }
        let (powered, connected) = read_bt_state();
        if let Some(i) = icon_weak.upgrade() {
            if !powered {
                i.set_icon_name(Some("bluetooth-disabled-symbolic"));
            } else if connected {
                i.set_icon_name(Some("bluetooth-active-symbolic"));
            } else {
                i.set_icon_name(Some("bluetooth-symbolic"));
            }
        }
        if let Some(c) = container_weak.upgrade() {
            c.remove_css_class("off");
            c.remove_css_class("connected");
            if !powered { c.add_css_class("off"); }
            else if connected { c.add_css_class("connected"); }
        }
        glib::ControlFlow::Continue
    });

    module.container
}

fn read_bt_state() -> (bool, bool) {
    let out = run_cmd(&["bluetoothctl", "show"]);
    let powered = out.contains("Powered: yes");
    let connected = {
        let devs = run_cmd(&["bluetoothctl", "devices", "Connected"]);
        !devs.trim().is_empty()
    };
    (powered, connected)
}

fn refresh_bt_list(list: &gtk4::Box) {
    remove_children(list);
    let devs = run_cmd(&["bluetoothctl", "devices", "Connected"]);
    if devs.trim().is_empty() {
        let lbl = gtk4::Label::new(Some("No connected devices"));
        lbl.set_xalign(0.0);
        list.append(&lbl);
        return;
    }
    for line in devs.lines() {
        // "Device XX:XX:XX:XX:XX:XX Name"
        let parts: Vec<&str> = line.splitn(3, ' ').collect();
        if parts.len() < 3 { continue; }
        let addr = parts[1].to_string();
        let name = parts[2].to_string();
        let row = gtk4::Button::with_label(&name);
        row.set_has_frame(false);
        row.add_css_class("bluetooth-device-row");
        let addr2 = addr.clone();
        row.connect_clicked(move |_| {
            run_detached_args("bluetoothctl", &["disconnect", &addr2]);
        });
        list.append(&row);
    }
}
