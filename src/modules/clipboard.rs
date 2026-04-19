use gtk4::prelude::*;
use glib::prelude::*;
use std::time::Duration;
use std::sync::{Arc, Mutex};
use super::helpers::*;

const MAX_HISTORY: usize = 50;

pub fn new_clipboard() -> gtk4::Widget {
    let btn = gtk4::Button::with_label("󰅇");
    btn.set_has_frame(false);
    btn.set_widget_name("clipboard");

    let (popup_clip, menu) = make_popup();
    menu.set_widget_name("clipboard-menu");

    let title = gtk4::Label::new(Some("Clipboard"));
    title.set_xalign(0.0);
    title.set_widget_name("clipboard-menu-title");
    menu.append(&title);

    let list_box = gtk4::Box::new(gtk4::Orientation::Vertical, 2);
    menu.append(&list_box);

    // Use Arc<Mutex<>> so it can be shared with the watcher thread
    let history: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));

    {
        let list_weak = list_box.downgrade();
        let history_clone = history.clone();
        attach_hover_popup(&btn, &popup_clip, move || {
            if let Some(list) = list_weak.upgrade() {
                let h = history_clone.lock().unwrap();
                rebuild_clipboard_list(&list, &h);
            }
        });
    }

    let popup_clip_weak = popup_clip.downgrade();
    btn.connect_clicked(move |_| {
        if let Some(p) = popup_clip_weak.upgrade() {
            if p.is_visible() { p.set_visible(false); } else { p.present(); }
        }
    });

    // Watch clipboard via wl-paste in a thread, send new entries via async_channel
    let (tx, rx) = async_channel::bounded::<String>(16);
    let history2 = history.clone();
    std::thread::spawn(move || {
        let mut child = match std::process::Command::new("wl-paste")
            .args(&["--watch", "cat"])
            .stdout(std::process::Stdio::piped())
            .spawn() {
            Ok(c) => c,
            Err(_) => return,
        };
        if let Some(stdout) = child.stdout.take() {
            use std::io::BufRead;
            for line in std::io::BufReader::new(stdout).lines() {
                if let Ok(text) = line {
                    let text = text.trim().to_string();
                    if text.is_empty() { continue; }
                    // Update history immediately in thread (Arc<Mutex>)
                    {
                        let mut h = history2.lock().unwrap();
                        h.retain(|e| e != &text);
                        h.insert(0, text.clone());
                        if h.len() > MAX_HISTORY { h.truncate(MAX_HISTORY); }
                    }
                    tx.send_blocking(text).ok();
                }
            }
        }
    });

    // We just use the receiver to trigger UI refresh if popup is open
    // (history is already updated in the thread via Arc<Mutex>)
    glib::MainContext::default().spawn_local(async move {
        while rx.recv().await.is_ok() {
            // No UI update needed here - popup rebuilds on open
        }
    });

    btn.upcast()
}

fn rebuild_clipboard_list(list: &gtk4::Box, history: &[String]) {
    remove_children(list);
    if history.is_empty() {
        let lbl = gtk4::Label::new(Some("(empty)"));
        lbl.set_xalign(0.0);
        list.append(&lbl);
        return;
    }
    for entry in history.iter().take(15) {
        let entry_clone = entry.clone();
        let display = truncate(entry, 60);
        let row = gtk4::Button::with_label(&display);
        row.set_has_frame(false);
        row.add_css_class("clipboard-row");
        row.connect_clicked(move |_| {
            let text = entry_clone.clone();
            std::thread::spawn(move || {
                let _ = std::process::Command::new("wl-copy")
                    .stdin(std::process::Stdio::null())
                    .arg(&text)
                    .spawn();
            });
        });
        list.append(&row);
    }
}
