use gtk4::prelude::*;
use glib::prelude::*;
use std::time::Duration;
use super::helpers::*;

pub fn new_power_profile() -> gtk4::Box {
    let module = TextModule::new("power-profiles-daemon");
    let container_weak = module.container.downgrade();
    let label_weak = module.label.downgrade();

    glib::timeout_add_local(Duration::from_secs(10), move || {
        if container_weak.upgrade().is_none() { return glib::ControlFlow::Break; }
        let profile = run_cmd(&["powerprofilesctl", "get"]);
        let text = match profile.as_str() {
            "performance" => "",
            "balanced" => "",
            "power-saver" => "",
            _ => "",
        };

        if let Some(label) = label_weak.upgrade() {
            label.set_label(text);
        }
        if let Some(c) = container_weak.upgrade() {
            let visible = !profile.is_empty();
            c.set_visible(visible);
            for cls in ["performance", "balanced", "power-saver"] { c.remove_css_class(cls); }
            if !profile.is_empty() {
                c.add_css_class(&profile);
                c.set_tooltip_text(Some(&profile));
            }
        }
        glib::ControlFlow::Continue
    });

    // Initial call
    let profile = run_cmd(&["powerprofilesctl", "get"]);
    let text = match profile.as_str() {
        "performance" => "",
        "balanced" => "",
        "power-saver" => "",
        _ => "",
    };
    module.label.set_label(text);
    module.container.set_visible(!profile.is_empty());

    module.container
}
