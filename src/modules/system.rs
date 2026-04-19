use gtk4::prelude::*;
use glib::prelude::*;
use std::time::Duration;
use super::helpers::*;

struct CpuSampler {
    last_idle: u64,
    last_total: u64,
    ready: bool,
}

impl CpuSampler {
    fn new() -> Self { Self { last_idle: 0, last_total: 0, ready: false } }

    fn read(&mut self) -> String {
        let stat = match std::fs::read_to_string("/proc/stat") {
            Ok(s) => s,
            Err(_) => return String::new(),
        };
        let first_line = stat.lines().next().unwrap_or("");
        let fields: Vec<&str> = first_line.split_whitespace().collect();
        if fields.len() < 5 { return String::new(); }

        let mut total: u64 = 0;
        let mut values: Vec<u64> = Vec::new();
        for f in &fields[1..] {
            let v: u64 = f.parse().unwrap_or(0);
            values.push(v);
            total += v;
        }
        let idle = values.get(3).copied().unwrap_or(0)
            + values.get(4).copied().unwrap_or(0);

        if !self.ready {
            self.last_idle = idle;
            self.last_total = total;
            self.ready = true;
            return "--%".to_string();
        }

        let total_delta = total.saturating_sub(self.last_total);
        let idle_delta = idle.saturating_sub(self.last_idle);
        self.last_idle = idle;
        self.last_total = total;

        if total_delta == 0 { return "--%".to_string(); }
        let usage = 100.0 * (total_delta - idle_delta) as f64 / total_delta as f64;
        format!("{:2.0}%", usage)
    }
}

pub fn new_cpu() -> gtk4::Box {
    let module = TextModule::new("cpu");
    module.container.set_spacing(4);

    let icon = gtk4::Image::from_icon_name("cpu-symbolic");
    icon.set_pixel_size(14);
    icon.add_css_class("module-icon");
    module.container.prepend(&icon);

    let sampler = std::rc::Rc::new(std::cell::RefCell::new(CpuSampler::new()));
    let container_weak = module.container.downgrade();
    let label_weak = module.label.downgrade();

    glib::timeout_add_local(Duration::from_millis(500), move || {
        if container_weak.upgrade().is_none() { return glib::ControlFlow::Break; }
        let text = sampler.borrow_mut().read();
        if let Some(l) = label_weak.upgrade() { l.set_label(&text); }
        if let Some(c) = container_weak.upgrade() { c.set_visible(!text.is_empty()); }
        glib::ControlFlow::Continue
    });

    module.container
}

pub fn new_memory() -> gtk4::Box {
    let module = TextModule::new("memory");
    module.container.set_spacing(4);

    let icon = gtk4::Image::from_icon_name("memory-symbolic");
    icon.set_pixel_size(14);
    icon.add_css_class("module-icon");
    module.container.prepend(&icon);

    let container_weak = module.container.downgrade();
    let label_weak = module.label.downgrade();

    glib::timeout_add_local(Duration::from_secs(3), move || {
        if container_weak.upgrade().is_none() { return glib::ControlFlow::Break; }
        let text = read_memory();
        if let Some(l) = label_weak.upgrade() { l.set_label(&text); }
        if let Some(c) = container_weak.upgrade() { c.set_visible(!text.is_empty()); }
        glib::ControlFlow::Continue
    });

    module.container
}

fn read_memory() -> String {
    let meminfo = match std::fs::read_to_string("/proc/meminfo") {
        Ok(s) => s,
        Err(_) => return String::new(),
    };
    let mut values: std::collections::HashMap<String, u64> = std::collections::HashMap::new();
    for line in meminfo.lines() {
        let fields: Vec<&str> = line.split_whitespace().collect();
        if fields.len() >= 2 {
            let key = fields[0].trim_end_matches(':').to_string();
            let val: u64 = fields[1].parse().unwrap_or(0);
            values.insert(key, val);
        }
    }
    let total = *values.get("MemTotal").unwrap_or(&0);
    let available = *values.get("MemAvailable").unwrap_or(&0);
    if total == 0 { return String::new(); }
    let used = total - available;
    let used_mb = used / 1024;
    let total_mb = total / 1024;
    format!("{}M", used_mb)
}

pub fn new_temperature() -> gtk4::Box {
    let module = TextModule::new("temperature");
    module.container.set_spacing(4);

    let icon = gtk4::Image::from_icon_name("temperature-symbolic");
    icon.set_pixel_size(14);
    icon.add_css_class("module-icon");
    module.container.prepend(&icon);

    let container_weak = module.container.downgrade();
    let label_weak = module.label.downgrade();

    glib::timeout_add_local(Duration::from_secs(5), move || {
        if container_weak.upgrade().is_none() { return glib::ControlFlow::Break; }
        let text = read_temperature();
        if let Some(l) = label_weak.upgrade() { l.set_label(&text); }
        if let Some(c) = container_weak.upgrade() {
            c.set_visible(!text.is_empty());
            // Add critical CSS class if >= 80°C
            if let Ok(t) = text.trim_end_matches('°').trim_end_matches('C').trim().parse::<f64>() {
                if t >= 80.0 { c.add_css_class("critical"); } else { c.remove_css_class("critical"); }
            }
        }
        glib::ControlFlow::Continue
    });

    module.container
}

fn read_temperature() -> String {
    let dir = match std::fs::read_dir("/sys/class/thermal") {
        Ok(d) => d,
        Err(_) => return String::new(),
    };

    let mut max_temp: f64 = f64::NEG_INFINITY;
    for entry in dir.flatten() {
        let path = entry.path();
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if !name_str.starts_with("thermal_zone") { continue; }
        let temp_path = path.join("temp");
        if let Ok(data) = std::fs::read_to_string(&temp_path) {
            if let Ok(val) = data.trim().parse::<f64>() {
                let temp = val / 1000.0;
                if temp > max_temp { max_temp = temp; }
            }
        }
    }

    if max_temp == f64::NEG_INFINITY { return String::new(); }
    format!("{:.0}°C", max_temp)
}
