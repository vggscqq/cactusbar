use gdk4::Monitor;
use gtk4::ApplicationWindow;
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};

pub fn init_layer_shell(window: &ApplicationWindow) {
    window.init_layer_shell();
    window.set_namespace(Some("statusbar"));
    window.set_layer(Layer::Top);
    window.set_anchor(Edge::Top, true);
    window.set_anchor(Edge::Left, true);
    window.set_anchor(Edge::Right, true);
    window.auto_exclusive_zone_enable();
    window.set_keyboard_mode(KeyboardMode::None);
}

pub fn set_layer_shell_monitor(window: &ApplicationWindow, monitor: &Monitor) {
    window.set_monitor(Some(monitor));
}
