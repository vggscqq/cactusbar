use gtk4::prelude::*;
use glib::prelude::*;
use std::time::Duration;
use super::helpers::*;

fn read_volume() -> (i32, bool) {
    let out = run_cmd(&["pactl", "get-sink-volume", "@DEFAULT_SINK@"]);
    let percent = out.split('%').next()
        .and_then(|s| s.split_whitespace().last())
        .and_then(|s| s.parse::<i32>().ok())
        .unwrap_or(0);
    let mute_out = run_cmd(&["pactl", "get-sink-mute", "@DEFAULT_SINK@"]);
    let muted = mute_out.contains("yes");
    (percent, muted)
}

fn volume_icon(percent: i32, muted: bool) -> &'static str {
    if muted || percent == 0 {
        "audio-volume-muted-symbolic"
    } else if percent < 35 {
        "audio-volume-low-symbolic"
    } else if percent < 70 {
        "audio-volume-medium-symbolic"
    } else {
        "audio-volume-high-symbolic"
    }
}

pub fn new_pipewire(cfg: &crate::config::Config) -> gtk4::Box {
    let module = TextModule::new("pipewire");
    remove_children(&module.container);
    module.container.set_visible(true);

    let icon_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 4);
    let vol_icon = gtk4::Image::from_icon_name("audio-volume-high-symbolic");
    icon_box.append(&vol_icon);

    let val_label = gtk4::Label::new(None);
    val_label.set_xalign(0.0);
    if cfg.audio.show_text {
        icon_box.append(&val_label);
    }
    module.container.append(&icon_box);

    let popover = gtk4::Popover::new();
    popover.add_css_class("status-popup");
    popover.set_has_arrow(false);
    popover.set_autohide(true);
    popover.set_parent(&module.container);

    let menu = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
    menu.set_widget_name("audio-menu");
    popover.set_child(Some(&menu));

    let menu_title = gtk4::Label::new(Some("Audio"));
    menu_title.set_widget_name("audio-menu-title");
    menu_title.set_xalign(0.0);
    menu.append(&menu_title);

    let show_text = cfg.audio.show_text;
    let on_click = cfg.audio.on_click.clone();
    if on_click.is_empty() {
        // no op
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
            glib::timeout_add_local_once(Duration::from_millis(100), move || { p.popdown(); });
        }
    });
    module.container.add_controller(motion);

    // Right click -> open mixer
    let on_click2 = on_click.clone();
    attach_click(&module.container,
        || {},
        move || { run_detached(&on_click2); }
    );

    // Scroll to change volume
    attach_scroll(&module.container,
        || { run_detached_args("wpctl", &["set-volume", "@DEFAULT_AUDIO_SINK@", "5%+"]); },
        || { run_detached_args("wpctl", &["set-volume", "@DEFAULT_AUDIO_SINK@", "5%-"]); }
    );

    let vol_icon_weak = vol_icon.downgrade();
    let val_label_weak = val_label.downgrade();
    let container_weak = module.container.downgrade();

    glib::timeout_add_local(Duration::from_secs(2), move || {
        if container_weak.upgrade().is_none() {
            return glib::ControlFlow::Break;
        }
        let (percent, muted) = read_volume();
        if let Some(icon) = vol_icon_weak.upgrade() {
            icon.set_icon_name(Some(volume_icon(percent, muted)));
        }
        if show_text {
            if let Some(lbl) = val_label_weak.upgrade() {
                lbl.set_label(&format!("{:3}%", percent));
            }
        }
        if let Some(c) = container_weak.upgrade() {
            if muted {
                c.add_css_class("muted");
            } else {
                c.remove_css_class("muted");
            }
        }
        glib::ControlFlow::Continue
    });

    module.container
}
