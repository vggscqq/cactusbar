use std::sync::OnceLock;
use tokio::sync::broadcast;
use tokio::io::AsyncBufReadExt;

static HYPR_TX: OnceLock<broadcast::Sender<String>> = OnceLock::new();

fn hypr_socket2_path() -> Option<String> {
    let sig = std::env::var("HYPRLAND_INSTANCE_SIGNATURE").ok()?;
    let xdg = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/run/user/1000".to_string());
    Some(format!("{}/hypr/{}/.socket2.sock", xdg, sig))
}

async fn read_hypr_events(tx: broadcast::Sender<String>) {
    loop {
        let path = match hypr_socket2_path() {
            Some(p) => p,
            None => {
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                continue;
            }
        };
        let stream = match tokio::net::UnixStream::connect(&path).await {
            Ok(s) => s,
            Err(_) => {
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                continue;
            }
        };
        let mut reader = tokio::io::BufReader::new(stream);
        let mut line = String::new();
        loop {
            line.clear();
            match reader.read_line(&mut line).await {
                Ok(0) => break,
                Ok(_) => {
                    let trimmed = line.trim().to_string();
                    if !trimmed.is_empty() {
                        let _ = tx.send(trimmed);
                    }
                }
                Err(_) => break,
            }
        }
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
}

pub fn hypr_subscribe() -> broadcast::Receiver<String> {
    let tx = HYPR_TX.get_or_init(|| {
        let (tx, _) = broadcast::channel(64);
        let tx_clone = tx.clone();
        crate::RUNTIME.get().unwrap().spawn(async move {
            read_hypr_events(tx_clone).await;
        });
        tx
    });
    tx.subscribe()
}

/// Returns an async_channel Receiver that delivers Hyprland events on any thread.
/// Attach to GTK main loop via: glib::MainContext::default().spawn_local(async move { while let Ok(ev) = rx.recv().await { ... } })
pub fn hypr_event_channel() -> async_channel::Receiver<String> {
    let (tx, rx) = async_channel::bounded::<String>(64);
    let mut sub = hypr_subscribe();
    crate::RUNTIME.get().unwrap().spawn(async move {
        loop {
            match sub.recv().await {
                Ok(event) => { let _ = tx.send(event).await; }
                Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
                Err(_) => break,
            }
        }
    });
    rx
}

/// Returns an async_channel Receiver for MPRIS events (polling every 2s).
pub fn mpris_event_channel() -> async_channel::Receiver<()> {
    let (tx, rx) = async_channel::bounded::<()>(16);
    crate::RUNTIME.get().unwrap().spawn(async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            let _ = tx.send(()).await;
        }
    });
    rx
}
