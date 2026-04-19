use gtk4::prelude::*;
use glib::prelude::*;
use std::time::Duration;
use super::helpers::*;

struct BatteryInfo {
    capacity: i32,
    icon_name: String,
    state: String,
    status: String,
    energy_now: f64,
    energy_full: f64,
    power_now: f64,
}

fn read_battery_info(device: &str) -> Option<BatteryInfo> {
    let base = format!("/sys/class/power_supply/{}", device);
    if !std::path::Path::new(&base).exists() {
        return None;
    }

    let capacity_str = read_first_existing(&[&format!("{}/capacity", base)]);
    let capacity = capacity_str.parse::<i32>().ok()?;

    let status = read_first_existing(&[&format!("{}/status", base)]).to_lowercase();
    let energy_now = read_first_existing(&[&format!("{}/energy_now", base)])
        .parse::<f64>().unwrap_or(0.0);
    let energy_full = read_first_existing(&[&format!("{}/energy_full", base)])
        .parse::<f64>().unwrap_or(0.0);
    let power_now = read_first_existing(&[&format!("{}/power_now", base)])
        .parse::<f64>().unwrap_or(0.0);

    let mut icon_name = match capacity {
        0..=9 => "battery-empty-symbolic",
        10..=29 => "battery-caution-symbolic",
        30..=54 => "battery-low-symbolic",
        55..=79 => "battery-good-symbolic",
        _ => "battery-full-symbolic",
    }.to_string();

    let state = if status == "charging" {
        icon_name = match capacity {
            0..=9 => "battery-empty-charging-symbolic",
            10..=29 => "battery-caution-charging-symbolic",
            30..=54 => "battery-low-charging-symbolic",
            55..=79 => "battery-good-charging-symbolic",
            _ => "battery-full-charging-symbolic",
        }.to_string();
        "charging".to_string()
    } else if status == "full" || status == "not charging" {
        icon_name = "battery-full-charged-symbolic".to_string();
        "plugged".to_string()
    } else if capacity <= 15 {
        "critical".to_string()
    } else {
        String::new()
    };

    Some(BatteryInfo { capacity, icon_name, state, status, energy_now, energy_full, power_now })
}

fn estimate_time(info: &BatteryInfo) -> String {
    if info.power_now <= 0.0 {
        return String::new();
    }
    match info.status.as_str() {
        "discharging" => {
            let hours = info.energy_now / info.power_now;
            format_duration(hours, "till empty")
        }
        "charging" => {
            let remaining = info.energy_full - info.energy_now;
            if remaining <= 0.0 { return String::new(); }
            let hours = remaining / info.power_now;
            format_duration(hours, "till full")
        }
        _ => String::new(),
    }
}

fn format_duration(hours: f64, suffix: &str) -> String {
    if hours <= 0.0 || hours.is_infinite() || hours.is_nan() {
        return String::new();
    }
    let total_min = (hours * 60.0).round() as i64;
    let h = total_min / 60;
    let m = total_min % 60;
    if h > 0 {
        format!("{}h {}m {}", h, m, suffix)
    } else {
        format!("{}m {}", m, suffix)
    }
}

pub fn new_battery(cfg: &crate::config::Config, device: &str, _name: &str) -> gtk4::Box {
    let show_text = cfg.battery_config.show_text;
    let device = device.to_string();

    let module = TextModule::new("battery");
    remove_children(&module.container);
    module.container.set_visible(true);

    let icon_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 4);
    let pct_label = gtk4::Label::new(None);
    pct_label.set_xalign(0.0);
    let batt_icon = gtk4::Image::from_icon_name("battery-full-symbolic");
    icon_box.append(&pct_label);
    icon_box.append(&batt_icon);
    module.container.append(&icon_box);

    let (popup, popup_box) = make_popup();
    popup_box.set_widget_name("battery-popup");

    let dev_label = gtk4::Label::new(None);
    dev_label.add_css_class("battery-popup-device");
    dev_label.set_xalign(0.0);
    let pct_popup = gtk4::Label::new(None);
    pct_popup.add_css_class("battery-popup-percent");
    pct_popup.set_xalign(0.0);
    let status_lbl = gtk4::Label::new(None);
    status_lbl.add_css_class("battery-popup-status");
    status_lbl.set_xalign(0.0);
    let time_lbl = gtk4::Label::new(None);
    time_lbl.add_css_class("battery-popup-time");
    time_lbl.set_xalign(0.0);

    popup_box.append(&dev_label);
    popup_box.append(&pct_popup);
    popup_box.append(&status_lbl);
    popup_box.append(&time_lbl);

    attach_hover_popup(&module.container, &popup, || {});

    let container_weak = module.container.downgrade();
    let icon_weak = batt_icon.downgrade();
    let pct_weak = pct_label.downgrade();
    let dev_weak = dev_label.downgrade();
    let pct_popup_weak = pct_popup.downgrade();
    let status_weak = status_lbl.downgrade();
    let time_weak = time_lbl.downgrade();
    let device2 = device.clone();

    glib::timeout_add_local(Duration::from_secs(10), move || {
        if container_weak.upgrade().is_none() {
            return glib::ControlFlow::Break;
        }
        let info = match read_battery_info(&device2) {
            Some(i) => i,
            None => {
                if let Some(c) = container_weak.upgrade() { c.set_visible(false); }
                return glib::ControlFlow::Continue;
            }
        };

        if let Some(icon) = icon_weak.upgrade() {
            icon.set_icon_name(Some(&info.icon_name));
        }
        if show_text {
            if let Some(l) = pct_weak.upgrade() {
                l.set_label(&format!("{}%", info.capacity));
            }
        }
        if let Some(c) = container_weak.upgrade() {
            for cls in ["charging", "plugged", "critical"] { c.remove_css_class(cls); }
            if !info.state.is_empty() { c.add_css_class(&info.state); }
        }
        if let Some(l) = dev_weak.upgrade() {
            l.set_label(&format!("Device: {}", device2));
        }
        if let Some(l) = pct_popup_weak.upgrade() {
            l.set_label(&format!("Capacity: {}%", info.capacity));
        }
        if let Some(l) = status_weak.upgrade() {
            let s = if info.status.is_empty() { "Unknown".to_string() } else {
                let mut s = info.status.clone();
                if let Some(c) = s.get_mut(0..1) { c.make_ascii_uppercase(); }
                s
            };
            l.set_label(&format!("Status: {}", s));
        }
        if let Some(l) = time_weak.upgrade() {
            let t = estimate_time(&info);
            l.set_label(&t);
            l.set_visible(!t.is_empty());
        }

        glib::ControlFlow::Continue
    });

    module.container
}
