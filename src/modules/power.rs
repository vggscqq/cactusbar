use gtk4::prelude::*;
use glib::prelude::*;
use std::time::Duration;
use super::helpers::*;

pub fn new_power() -> gtk4::Button {
    let btn = gtk4::Button::with_label("⏻");
    btn.set_has_frame(false);
    btn.set_widget_name("custom-power");

    let (popup_power, list) = make_popup();
    list.set_widget_name("power-menu");

    let actions: &[(&str, &str, &[&str])] = &[
        ("Shutdown", "poweroff", &[]),
        ("Reboot", "reboot", &[]),
        ("Suspend", "systemctl", &["suspend"]),
        ("Hibernate", "systemctl", &["hibernate"]),
    ];

    for (label, cmd, args) in actions {
        let item = gtk4::Button::with_label(label);
        item.set_has_frame(false);
        let popup_weak = popup_power.downgrade();
        let cmd = cmd.to_string();
        let args: Vec<String> = args.iter().map(|s| s.to_string()).collect();
        item.connect_clicked(move |_| {
            if let Some(p) = popup_weak.upgrade() { p.set_visible(false); }
            let args_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
            run_detached_args(&cmd, &args_refs);
        });
        list.append(&item);
    }

    attach_hover_popup(&btn, &popup_power, || {});

    btn
}
