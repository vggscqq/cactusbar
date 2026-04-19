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

/// Create a layer-shell overlay window to use as a hover popup.
///
/// Returns `(window, content_box)`.  Add children to `content_box`.
pub fn make_popup() -> (gtk4::Window, gtk4::Box) {
    use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};

    let mut builder = gtk4::Window::builder()
        .decorated(false)
        .resizable(false);

    if let Some(app) = crate::APP.with(|a| a.borrow().clone()) {
        builder = builder.application(&app);
    }

    let popup = builder.build();
    popup.init_layer_shell();
    popup.set_layer(Layer::Overlay);
    popup.set_anchor(Edge::Top, true);
    popup.set_anchor(Edge::Left, true);
    popup.set_exclusive_zone(-1);
    popup.set_keyboard_mode(KeyboardMode::None);
    popup.add_css_class("popup-window");

    let menu = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
    popup.set_child(Some(&menu));

    (popup, menu)
}

/// Position `popup` just below `anchor` on screen.
fn position_popup_below(popup: &gtk4::Window, anchor: &gtk4::Widget) {
    use gtk4_layer_shell::{Edge, LayerShell};

    let bar_win = match anchor
        .ancestor(gtk4::Window::static_type())
        .and_then(|o| o.downcast::<gtk4::Widget>().ok())
    {
        Some(w) => w,
        None => { popup.set_margin(Edge::Top, 34); return; }
    };

    let bar_h = bar_win.height().max(1);

    let x = anchor
        .compute_bounds(&bar_win)
        .map(|b| b.x().max(0.0) as i32)
        .unwrap_or(0);

    popup.set_margin(Edge::Left, x);
    popup.set_margin(Edge::Top, bar_h);
}

/// Attach a hover-triggered layer-shell popup window to `anchor`.
///
/// `on_open` is called each time the pointer enters the anchor (use it to
/// refresh popup contents).  Pass `|| {}` when no refresh is needed.
///
/// The popup stays open while the pointer is over either the anchor or the
/// popup itself; it closes 150 ms after the pointer leaves both regions.
///
/// The popup window is kept alive by the motion-controller closure tied to
/// `anchor`; no external reference needs to be held.
pub fn attach_hover_popup<W, F>(anchor: &W, popup: &gtk4::Window, on_open: F)
where
    W: IsA<gtk4::Widget>,
    F: Fn() + 'static,
{
    let popup_weak = popup.downgrade();
    let over_anchor = std::rc::Rc::new(std::cell::Cell::new(false));
    let over_popup  = std::rc::Rc::new(std::cell::Cell::new(false));

    // ── anchor motion ────────────────────────────────────────────────────────
    let anchor_motion = gtk4::EventControllerMotion::new();
    {
        // Hold a strong GObject reference inside this closure so the window
        // lives exactly as long as the anchor widget does.
        let popup_strong = popup.clone();
        let pw = popup_weak.clone();
        let oa = over_anchor.clone();
        anchor_motion.connect_enter(move |ctrl, _, _| {
            let _ = &popup_strong; // keep window alive
            oa.set(true);
            on_open();
            if let Some(p) = pw.upgrade() {
                if let Some(widget) = ctrl.widget() {
                    position_popup_below(&p, &widget);
                }
                p.present();
            }
        });
    }
    {
        let oa = over_anchor.clone();
        let op = over_popup.clone();
        let pw = popup_weak.clone();
        anchor_motion.connect_leave(move |_| {
            oa.set(false);
            let oa2 = oa.clone(); let op2 = op.clone(); let pw2 = pw.clone();
            glib::timeout_add_local_once(Duration::from_millis(150), move || {
                if !oa2.get() && !op2.get() {
                    if let Some(p) = pw2.upgrade() { p.set_visible(false); }
                }
            });
        });
    }
    anchor.add_controller(anchor_motion);

    // ── popup motion ─────────────────────────────────────────────────────────
    let popup_motion = gtk4::EventControllerMotion::new();
    {
        let op = over_popup.clone();
        popup_motion.connect_enter(move |_, _, _| { op.set(true); });
    }
    {
        let oa = over_anchor.clone();
        let op = over_popup.clone();
        let pw = popup_weak.clone();
        popup_motion.connect_leave(move |_| {
            op.set(false);
            let oa2 = oa.clone(); let op2 = op.clone(); let pw2 = pw.clone();
            glib::timeout_add_local_once(Duration::from_millis(150), move || {
                if !oa2.get() && !op2.get() {
                    if let Some(p) = pw2.upgrade() { p.set_visible(false); }
                }
            });
        });
    }
    popup.add_controller(popup_motion);
}
