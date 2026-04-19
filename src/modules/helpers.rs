use gtk4::prelude::*;
use std::time::Duration;

pub struct TextModule {
    pub container: gtk4::Box,
    pub label: gtk4::Label,
}

impl TextModule {
    pub fn new(css_name: &str) -> Self {
        let container = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        container.set_widget_name(css_name);
        container.add_css_class("module");
        container.set_visible(false);

        let label = gtk4::Label::new(None);
        container.append(&label);

        Self { container, label }
    }

    pub fn set_text(&self, text: &str) {
        let text = text.trim();
        self.label.set_label(text);
        self.container.set_visible(!text.is_empty());
    }

    pub fn set_visible(&self, v: bool) {
        self.container.set_visible(v);
    }
}

pub fn run_cmd(args: &[&str]) -> String {
    if args.is_empty() {
        return String::new();
    }
    use std::sync::mpsc;
    let (tx, rx) = mpsc::channel();
    let args_owned: Vec<String> = args.iter().map(|s| s.to_string()).collect();
    std::thread::spawn(move || {
        let result = std::process::Command::new(&args_owned[0])
            .args(&args_owned[1..])
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_default();
        let _ = tx.send(result);
    });
    rx.recv_timeout(Duration::from_millis(1500))
        .unwrap_or_default()
}

pub fn run_detached(cmd: &str) {
    let parts: Vec<String> = cmd.split_whitespace().map(|s| s.to_string()).collect();
    if parts.is_empty() {
        return;
    }
    let _ = std::process::Command::new(&parts[0])
        .args(&parts[1..])
        .spawn();
}

pub fn run_detached_args(cmd: &str, args: &[&str]) {
    let _ = std::process::Command::new(cmd).args(args).spawn();
}

pub fn attach_click<W: IsA<gtk4::Widget>>(
    widget: &W,
    left: impl Fn() + 'static,
    right: impl Fn() + 'static,
) {
    let gesture = gtk4::GestureClick::new();
    gesture.set_button(0);
    gesture.connect_pressed(move |g, _n, _x, _y| {
        match g.current_button() {
            1 => left(),
            3 => right(),
            _ => {}
        }
    });
    widget.add_controller(gesture);
}

pub fn attach_click_opt<W: IsA<gtk4::Widget>>(
    widget: &W,
    left: Option<Box<dyn Fn()>>,
    right: Option<Box<dyn Fn()>>,
) {
    let gesture = gtk4::GestureClick::new();
    gesture.set_button(0);
    gesture.connect_pressed(move |g, _n, _x, _y| {
        match g.current_button() {
            1 => {
                if let Some(f) = &left {
                    f();
                }
            }
            3 => {
                if let Some(f) = &right {
                    f();
                }
            }
            _ => {}
        }
    });
    widget.add_controller(gesture);
}

pub fn attach_scroll<W: IsA<gtk4::Widget>>(
    widget: &W,
    on_up: impl Fn() + 'static,
    on_down: impl Fn() + 'static,
) {
    let scroll = gtk4::EventControllerScroll::new(gtk4::EventControllerScrollFlags::VERTICAL);
    scroll.connect_scroll(move |_, _dx, dy| {
        if dy < 0.0 {
            on_up();
        } else if dy > 0.0 {
            on_down();
        }
        glib::Propagation::Stop
    });
    widget.add_controller(scroll);
}

pub fn remove_children(box_: &gtk4::Box) {
    while let Some(child) = box_.first_child() {
        box_.remove(&child);
    }
}

pub fn read_first_existing(paths: &[&str]) -> String {
    for path in paths {
        if let Ok(data) = std::fs::read_to_string(path) {
            return data.trim().to_string();
        }
    }
    String::new()
}

pub fn fetch_json(url: &str) -> Option<serde_json::Value> {
    ureq::get(url)
        .set("User-Agent", "cactusbar/1.0")
        .call()
        .ok()
        .and_then(|r| r.into_json::<serde_json::Value>().ok())
}

pub fn clamp(v: f64, min: f64, max: f64) -> f64 {
    v.max(min).min(max)
}

pub fn truncate(s: &str, n: usize) -> String {
    let chars: Vec<char> = s.chars().collect();
    if chars.len() <= n {
        s.to_string()
    } else {
        chars[..n].iter().collect::<String>() + "…"
    }
}

pub fn marquee_text(text: &str, max_len: usize, step: usize) -> String {
    let chars: Vec<char> = text.chars().collect();
    if chars.len() <= max_len {
        return text.to_string();
    }
    let idx = step % chars.len();
    let rotated: Vec<char> = chars[idx..].iter().chain(chars[..idx].iter()).cloned().collect();
    rotated[..max_len.min(rotated.len())].iter().collect()
}

pub fn start_polling_weak<W, F>(widget: &W, interval_ms: u64, mut f: F)
where
    W: IsA<gtk4::Widget>,
    F: FnMut(&W) + 'static,
{
    f(widget);
    let weak = widget.downgrade();
    glib::timeout_add_local(Duration::from_millis(interval_ms), move || {
        match weak.upgrade() {
            Some(w) => {
                f(&w);
                glib::ControlFlow::Continue
            }
            None => glib::ControlFlow::Break,
        }
    });
}

pub fn make_popup(anchor: &impl IsA<gtk4::Widget>) -> (gtk4::Popover, gtk4::Box) {
    let popover = gtk4::Popover::new();
    popover.add_css_class("status-popup");
    popover.set_has_arrow(false);
    popover.set_autohide(true);
    popover.set_parent(anchor);

    let menu = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
    popover.set_child(Some(&menu));

    (popover, menu)
}

pub fn attach_hover_popup<W>(anchor: &W, popover: gtk4::Popover)
where
    W: IsA<gtk4::Widget>,
{
    let popover_weak = popover.downgrade();
    let motion = gtk4::EventControllerMotion::new();
    {
        let popover_weak = popover_weak.clone();
        motion.connect_enter(move |_, _x, _y| {
            if let Some(p) = popover_weak.upgrade() {
                p.popup();
            }
        });
    }
    {
        let popover_weak = popover_weak.clone();
        motion.connect_leave(move |_| {
            if let Some(p) = popover_weak.upgrade() {
                glib::timeout_add_local_once(Duration::from_millis(100), move || {
                    p.popdown();
                });
            }
        });
    }
    anchor.add_controller(motion);
}
