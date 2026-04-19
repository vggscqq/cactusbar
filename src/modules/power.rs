use gtk4::prelude::*;
use glib::prelude::*;
use std::time::Duration;
use super::helpers::*;

pub fn new_power() -> gtk4::Button {
    let btn = gtk4::Button::with_label("⏻");
    btn.set_has_frame(false);
    btn.set_widget_name("custom-power");

    let popover = gtk4::Popover::new();
    popover.add_css_class("status-popup");
    popover.set_has_arrow(false);
    popover.set_autohide(true);
    popover.set_parent(&btn);

    let list = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
    list.set_widget_name("power-menu");
    popover.set_child(Some(&list));

    let actions: &[(&str, &str, &[&str])] = &[
        ("Shutdown", "poweroff", &[]),
        ("Reboot", "reboot", &[]),
        ("Suspend", "systemctl", &["suspend"]),
        ("Hibernate", "systemctl", &["hibernate"]),
    ];

    for (label, cmd, args) in actions {
        let item = gtk4::Button::with_label(label);
        item.set_has_frame(false);
        let popover_weak = popover.downgrade();
        let cmd = cmd.to_string();
        let args: Vec<String> = args.iter().map(|s| s.to_string()).collect();
        item.connect_clicked(move |_| {
            if let Some(p) = popover_weak.upgrade() { p.popdown(); }
            let args_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
            run_detached_args(&cmd, &args_refs);
        });
        list.append(&item);
    }

    // Hover popup
    let motion = gtk4::EventControllerMotion::new();
    let popover_weak = popover.downgrade();
    motion.connect_enter(move |_, _, _| {
        if let Some(p) = popover_weak.upgrade() { p.popup(); }
    });
    let popover_weak2 = popover.downgrade();
    motion.connect_leave(move |_| {
        if let Some(p) = popover_weak2.upgrade() {
            glib::timeout_add_local_once(Duration::from_millis(150), move || { p.popdown(); });
        }
    });
    btn.add_controller(motion);

    btn
}
