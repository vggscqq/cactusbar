use gtk4::prelude::*;
use glib::prelude::*;
use std::time::Duration;
use super::helpers::*;

fn notification_icon(class: &str) -> &'static str {
    match class {
        "dnd-notification" | "dnd-none" | "dnd-inhibited-notification" | "dnd-inhibited-none" => "󰂛",
        _ => "󰂚",
    }
}

fn read_notification_state() -> (String, i32) {
    let count_out = run_cmd(&["swaync-client", "--skip-wait", "--count"]);
    let count: i32 = count_out.trim().parse().unwrap_or(0);
    let dnd = {
        let out = run_cmd(&["swaync-client", "--skip-wait", "--get-dnd"]);
        let v = out.trim().to_lowercase();
        v == "true" || v == "1"
    };
    let inhibited = {
        let out = run_cmd(&["swaync-client", "--skip-wait", "--get-inhibited"]);
        let v = out.trim().to_lowercase();
        v == "true" || v == "1"
    };
    let class = if dnd {
        if count > 0 { "dnd-notification" } else { "dnd-none" }
    } else if inhibited {
        if count > 0 { "inhibited-notification" } else { "inhibited-none" }
    } else if count > 0 { "notification" } else { "none" };
    (class.to_string(), count)
}

pub fn new_notification() -> gtk4::Box {
    let module = TextModule::new("custom-notification");

    let container_weak = module.container.downgrade();
    let label_weak = module.label.downgrade();

    let refresh = {
        let container_weak = container_weak.clone();
        let label_weak = label_weak.clone();
        move || {
            let (class, count) = read_notification_state();
            let text = format!("{}{}", notification_icon(&class), count);
            if let Some(l) = label_weak.upgrade() { l.set_label(&text); }
            if let Some(c) = container_weak.upgrade() {
                c.set_visible(true);
                if class.starts_with("dnd") { c.add_css_class("dnd"); } else { c.remove_css_class("dnd"); }
            }
        }
    };

    refresh();

    attach_click(&module.container,
        || { run_detached_args("swaync-client", &["-t", "-sw"]); },
        || { run_detached_args("swaync-client", &["-d", "-sw"]); }
    );

    // Subscribe to swaync via async_channel
    let (tx, rx) = async_channel::bounded::<()>(8);
    std::thread::spawn(move || {
        loop {
            let child = std::process::Command::new("swaync-client")
                .args(&["--skip-wait", "--subscribe-waybar"])
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::null())
                .spawn();
            let mut child = match child {
                Ok(c) => c,
                Err(_) => { std::thread::sleep(Duration::from_secs(3)); continue; }
            };
            if let Some(stdout) = child.stdout.take() {
                use std::io::BufRead;
                for _ in std::io::BufReader::new(stdout).lines() {
                    let _ = tx.send_blocking(());
                }
            }
            let _ = child.wait();
            std::thread::sleep(Duration::from_secs(2));
        }
    });

    let refresh2 = refresh.clone();
    glib::MainContext::default().spawn_local(async move {
        while rx.recv().await.is_ok() {
            refresh2();
        }
    });

    let module_weak = module.container.downgrade();
    glib::timeout_add_local(Duration::from_secs(30), move || {
        if module_weak.upgrade().is_none() { return glib::ControlFlow::Break; }
        refresh();
        glib::ControlFlow::Continue
    });

    module.container
}
