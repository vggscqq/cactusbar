use gtk4::prelude::*;
use glib::prelude::*;
use std::time::Duration;
use super::helpers::*;

// ── Volume helpers ────────────────────────────────────────────────────────────

fn parse_volume_str(s: &str) -> i32 {
    // Works on both "get-sink-volume" output and "pactl list sinks" Volume: lines.
    // Finds the first "NNN%" token.
    s.split('%').next()
        .and_then(|p| p.split_whitespace().last())
        .and_then(|p| p.parse::<i32>().ok())
        .unwrap_or(0)
}

fn read_volume() -> (i32, bool) {
    let out = run_cmd(&["pactl", "get-sink-volume", "@DEFAULT_SINK@"]);
    let mute_out = run_cmd(&["pactl", "get-sink-mute", "@DEFAULT_SINK@"]);
    (parse_volume_str(&out), mute_out.contains("yes"))
}

fn volume_icon(percent: i32, muted: bool) -> &'static str {
    if muted || percent == 0 { "audio-volume-muted-symbolic" }
    else if percent < 35     { "audio-volume-low-symbolic" }
    else if percent < 70     { "audio-volume-medium-symbolic" }
    else                     { "audio-volume-high-symbolic" }
}

// ── Device listing ────────────────────────────────────────────────────────────

#[derive(Clone)]
struct AudioDevice {
    name:        String,
    description: String,
    volume:      i32,
    muted:       bool,
}

fn parse_devices(output: &str, entry_prefix: &str) -> Vec<AudioDevice> {
    let mut devices: Vec<AudioDevice> = Vec::new();
    let marker = format!("{} #", entry_prefix);
    let mut in_dev = false;
    let mut name = String::new();
    let mut desc = String::new();
    let mut volume = 0i32;
    let mut muted = false;

    for line in output.lines() {
        let t = line.trim();
        if t.starts_with(&marker) {
            if in_dev && !name.is_empty() {
                devices.push(AudioDevice {
                    name: name.clone(),
                    description: if desc.is_empty() { name.clone() } else { desc.clone() },
                    volume, muted,
                });
            }
            in_dev = true;
            name.clear(); desc.clear(); volume = 0; muted = false;
        } else if in_dev {
            if let Some(v) = t.strip_prefix("Name: ")        { name   = v.to_string(); }
            else if let Some(v) = t.strip_prefix("Description: ") { desc = v.to_string(); }
            else if t.starts_with("Volume: ")                { volume = parse_volume_str(t); }
            else if let Some(v) = t.strip_prefix("Mute: ")   { muted  = v.trim() == "yes"; }
        }
    }
    if in_dev && !name.is_empty() {
        devices.push(AudioDevice {
            name,
            description: if desc.is_empty() { "Unknown".to_string() } else { desc },
            volume, muted,
        });
    }
    devices
}

fn get_sinks() -> (Vec<AudioDevice>, String) {
    let output  = run_cmd(&["pactl", "list", "sinks"]);
    let default = run_cmd(&["pactl", "get-default-sink"]);
    (parse_devices(&output, "Sink"), default.trim().to_string())
}

fn get_sources() -> (Vec<AudioDevice>, String) {
    let output  = run_cmd(&["pactl", "list", "sources"]);
    let default = run_cmd(&["pactl", "get-default-source"]);
    let mut devices = parse_devices(&output, "Source");
    devices.retain(|d| !d.name.ends_with(".monitor"));
    (devices, default.trim().to_string())
}

// ── Popup content ─────────────────────────────────────────────────────────────

fn build_device_row(device: &AudioDevice, is_default: bool, is_sink: bool) -> gtk4::Box {
    let row = gtk4::Box::new(gtk4::Orientation::Vertical, 3);
    row.add_css_class("audio-device-row");

    // ── name line ─────────────────────────────────────────────────────────────
    let name_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 4);
    if is_default {
        let check = gtk4::Image::from_icon_name("object-select-symbolic");
        check.set_pixel_size(10);
        name_row.append(&check);
    }
    let desc_lbl = gtk4::Label::new(Some(&truncate(&device.description, 32)));
    desc_lbl.set_xalign(0.0);
    desc_lbl.add_css_class("audio-device-name");
    if is_default { desc_lbl.add_css_class("audio-device-default"); }
    name_row.append(&desc_lbl);
    row.append(&name_row);

    // Click non-default device to set it as default
    if !is_default {
        let dev_name = device.name.clone();
        attach_click(&name_row, move || {
            if is_sink {
                run_detached_args("pactl", &["set-default-sink",   &dev_name]);
            } else {
                run_detached_args("pactl", &["set-default-source", &dev_name]);
            }
        }, || {});
        name_row.add_css_class("audio-device-clickable");
    }

    // ── volume line ───────────────────────────────────────────────────────────
    let vol_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 4);

    let mute_btn = gtk4::Button::from_icon_name(volume_icon(device.volume, device.muted));
    mute_btn.set_has_frame(false);
    mute_btn.set_valign(gtk4::Align::Center);
    mute_btn.add_css_class("audio-mute-btn");
    vol_row.append(&mute_btn);

    let scale = gtk4::Scale::with_range(gtk4::Orientation::Horizontal, 0.0, 150.0, 1.0);
    scale.set_value(device.volume as f64);
    scale.set_draw_value(false);
    scale.set_hexpand(true);
    scale.set_valign(gtk4::Align::Center);
    scale.set_size_request(180, -1);
    scale.add_css_class("audio-slider");
    scale.add_mark(100.0, gtk4::PositionType::Bottom, None);
    if device.muted { scale.set_sensitive(false); scale.add_css_class("audio-slider-muted"); }
    vol_row.append(&scale);

    let pct_lbl = gtk4::Label::new(Some(&format!("{}%", device.volume)));
    pct_lbl.set_size_request(46, -1);
    pct_lbl.set_xalign(1.0);
    pct_lbl.set_valign(gtk4::Align::Center);
    pct_lbl.add_css_class("audio-vol-pct");
    vol_row.append(&pct_lbl);

    row.append(&vol_row);

    // ── wire slider ───────────────────────────────────────────────────────────
    let dev_name_s = device.name.clone();
    let pct_weak   = pct_lbl.downgrade();
    let mute_weak  = mute_btn.downgrade();
    scale.connect_value_changed(move |s| {
        let val = s.value().round() as i32;
        if let Some(l) = pct_weak.upgrade()  { l.set_label(&format!("{}%", val)); }
        if let Some(b) = mute_weak.upgrade() { b.set_icon_name(volume_icon(val, false)); }
        let pct = format!("{}%", val);
        if is_sink {
            run_detached_args("pactl", &["set-sink-volume",   &dev_name_s, &pct]);
        } else {
            run_detached_args("pactl", &["set-source-volume", &dev_name_s, &pct]);
        }
    });

    // ── wire mute button ──────────────────────────────────────────────────────
    let dev_name_m  = device.name.clone();
    let scale_weak  = scale.downgrade();
    let pct_weak2   = pct_lbl.downgrade();
    let muted_cell  = std::rc::Rc::new(std::cell::Cell::new(device.muted));
    mute_btn.connect_clicked(move |btn| {
        let now_muted = !muted_cell.get();
        muted_cell.set(now_muted);
        if let Some(s) = scale_weak.upgrade() {
            s.set_sensitive(!now_muted);
            if now_muted { s.add_css_class("audio-slider-muted"); } else { s.remove_css_class("audio-slider-muted"); }
            let vol = s.value().round() as i32;
            btn.set_icon_name(volume_icon(vol, now_muted));
            if let Some(l) = pct_weak2.upgrade() { l.set_label(&format!("{}%", vol)); }
        }
        let mute_str = if now_muted { "1" } else { "0" };
        if is_sink {
            run_detached_args("pactl", &["set-sink-mute",   &dev_name_m, mute_str]);
        } else {
            run_detached_args("pactl", &["set-source-mute", &dev_name_m, mute_str]);
        }
    });

    row
}

fn refresh_audio_menu(menu: &gtk4::Box) {
    remove_children(menu);

    let title = gtk4::Label::new(Some("Audio"));
    title.set_widget_name("audio-menu-title");
    title.set_xalign(0.0);
    menu.append(&title);

    let (sinks, default_sink)     = get_sinks();
    let (sources, default_source) = get_sources();

    // Outputs
    let out_lbl = gtk4::Label::new(Some("OUTPUTS"));
    out_lbl.add_css_class("audio-section-label");
    out_lbl.set_xalign(0.0);
    out_lbl.set_valign(gtk4::Align::Center);
    menu.append(&out_lbl);
    if sinks.is_empty() {
        let none = gtk4::Label::new(Some("No outputs found"));
        none.set_xalign(0.0);
        none.add_css_class("audio-empty");
        menu.append(&none);
    } else {
        for sink in &sinks {
            menu.append(&build_device_row(sink, sink.name == default_sink, true));
        }
    }

    // Inputs
    let in_lbl = gtk4::Label::new(Some("INPUTS"));
    in_lbl.add_css_class("audio-section-label");
    in_lbl.set_xalign(0.0);
    in_lbl.set_valign(gtk4::Align::Center);
    menu.append(&in_lbl);
    if sources.is_empty() {
        let none = gtk4::Label::new(Some("No inputs found"));
        none.set_xalign(0.0);
        none.add_css_class("audio-empty");
        menu.append(&none);
    } else {
        for source in &sources {
            menu.append(&build_device_row(source, source.name == default_source, false));
        }
    }
}

// ── Module widget ─────────────────────────────────────────────────────────────

pub fn new_pipewire(cfg: &crate::config::Config) -> gtk4::Box {
    let module = TextModule::new("pipewire");
    remove_children(&module.container);
    module.container.set_visible(true);

    let icon_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 4);
    let vol_icon = gtk4::Image::from_icon_name("audio-volume-high-symbolic");
    icon_box.append(&vol_icon);

    let val_label = gtk4::Label::new(None);
    val_label.set_xalign(0.0);
    icon_box.append(&val_label);
    module.container.append(&icon_box);

    let (popup, menu) = make_popup();
    menu.set_widget_name("audio-menu");
    menu.set_spacing(6);
    menu.set_size_request(300, -1);

    {
        let menu_weak = menu.downgrade();
        attach_hover_popup(&module.container, &popup, move || {
            if let Some(m) = menu_weak.upgrade() { refresh_audio_menu(&m); }
        });
    }

    let on_click = cfg.audio.on_click.clone();
    attach_click(&module.container,
        || {},
        move || { run_detached(&on_click); }
    );

    attach_scroll(&module.container,
        || { run_detached_args("wpctl", &["set-volume", "@DEFAULT_AUDIO_SINK@", "5%+"]); },
        || { run_detached_args("wpctl", &["set-volume", "@DEFAULT_AUDIO_SINK@", "5%-"]); }
    );

    let vol_icon_weak  = vol_icon.downgrade();
    let val_label_weak = val_label.downgrade();
    let container_weak = module.container.downgrade();

    glib::timeout_add_local(Duration::from_secs(2), move || {
        if container_weak.upgrade().is_none() { return glib::ControlFlow::Break; }
        let (percent, muted) = read_volume();
        if let Some(icon) = vol_icon_weak.upgrade() {
            icon.set_icon_name(Some(volume_icon(percent, muted)));
        }
        if let Some(lbl) = val_label_weak.upgrade() {
            lbl.set_label(&format!("{:3}%", percent));
        }
        if let Some(c) = container_weak.upgrade() {
            if muted { c.add_css_class("muted"); } else { c.remove_css_class("muted"); }
        }
        glib::ControlFlow::Continue
    });

    module.container
}
