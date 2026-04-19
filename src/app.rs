use gtk4::prelude::*;
use glib::prelude::*;

const DEFAULT_CSS: &str = include_str!("../styles/bar.css");

pub fn new(
    app: &gtk4::Application,
    css_path: &str,
    monitor: Option<&gdk4::Monitor>,
    monitor_name: &str,
) -> gtk4::ApplicationWindow {
    load_css(css_path);
    let cfg = crate::config::load();

    let window = gtk4::ApplicationWindow::new(app);
    window.set_title(Some("Status Bar"));
    window.set_decorated(false);
    window.set_deletable(false);
    window.set_resizable(false);
    window.set_default_size(1920, 34);
    window.set_widget_name("status-bar");

    crate::layershell::init_layer_shell(&window);
    if let Some(mon) = monitor {
        crate::layershell::set_layer_shell_monitor(&window, mon);
    }

    let root = gtk4::CenterBox::new();

    let left = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
    left.add_css_class("modules-left");

    let center = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
    center.add_css_class("modules-center");

    let right = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
    right.add_css_class("modules-right");

    root.set_start_widget(Some(&left));
    root.set_center_widget(Some(&center));
    root.set_end_widget(Some(&right));
    window.set_child(Some(&root));

    let mods = &cfg.modules;

    if mods.workspaces {
        left.append(&crate::modules::workspaces::new_workspaces(monitor_name));
    }
    if mods.focused_app {
        left.append(&crate::modules::focused_app::new_focused_app(&cfg, &window, monitor_name));
    }
    if mods.music {
        left.append(&crate::modules::media::new_music());
    }
    if mods.mode {
        left.append(&crate::modules::input::new_mode());
    }
    if mods.scratchpad {
        left.append(&crate::modules::input::new_scratchpad());
    }
    if mods.date_clock {
        center.append(&crate::modules::clocks::new_date_clock());
    }
    if mods.time_clock {
        center.append(&crate::modules::clocks::new_time_clock(&cfg));
    }
    if mods.notification {
        center.append(&crate::modules::notifications::new_notification());
    }
    if mods.mpd {
        right.append(&crate::modules::media::new_mpd());
    }
    if mods.wallpaper {
        right.append(&crate::modules::wallpaper::new_wallpaper(&cfg));
    }
    if mods.clipboard {
        right.append(&crate::modules::clipboard::new_clipboard());
    }
    if mods.weather {
        right.append(&crate::modules::weather::new_weather(&cfg));
    }
    if mods.pipewire {
        right.append(&crate::modules::audio::new_pipewire(&cfg));
    }
    if mods.bluetooth {
        right.append(&crate::modules::bluetooth::new_bluetooth(&cfg));
    }
    if mods.network {
        right.append(&crate::modules::network::new_network(&cfg));
    }
    if mods.power_profile {
        right.append(&crate::modules::power_profile::new_power_profile());
    }
    if mods.cpu {
        right.append(&crate::modules::system::new_cpu());
    }
    if mods.memory {
        right.append(&crate::modules::system::new_memory());
    }
    if mods.temperature {
        right.append(&crate::modules::system::new_temperature());
    }
    if mods.keyboard_state {
        right.append(&crate::modules::input::new_keyboard_state());
    }
    if mods.language {
        right.append(&crate::modules::input::new_language(&cfg));
    }
    if mods.battery {
        right.append(&crate::modules::battery::new_battery(&cfg, "BAT0", "battery"));
    }
    if mods.tray {
        right.append(&crate::modules::tray::new_tray());
    }
    if mods.power {
        right.append(crate::modules::power::new_power().upcast_ref::<gtk4::Widget>());
    }

    window
}

fn load_css(css_path: &str) {
    let display = match gdk4::Display::default() {
        Some(d) => d,
        None => return,
    };

    let css = if !css_path.is_empty() {
        match std::fs::read_to_string(css_path) {
            Ok(data) => {
                log::info!("loaded css from flag: {}", css_path);
                data
            }
            Err(e) => {
                log::warn!("--css flag path unreadable, falling back: {}", e);
                load_user_or_default_css()
            }
        }
    } else {
        load_user_or_default_css()
    };

    if css.is_empty() {
        return;
    }

    let provider = gtk4::CssProvider::new();
    provider.load_from_string(&css);
    gtk4::style_context_add_provider_for_display(
        &display,
        &provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}

fn load_user_or_default_css() -> String {
    if let Ok(home) = std::env::var("HOME") {
        let path = format!("{}/.config/cactusbar/style.css", home);
        if let Ok(data) = std::fs::read_to_string(&path) {
            log::info!("loaded css from {}", path);
            return data;
        }
    }
    DEFAULT_CSS.to_string()
}
