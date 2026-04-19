use async_channel;
use std::sync::{Arc, Mutex};
use zbus::interface;

#[derive(Clone, Debug)]
pub enum SniEvent {
    ItemRegistered(String),
    ItemUnregistered(String),
}

#[derive(Clone)]
struct SniWatcher {
    items: Arc<Mutex<Vec<String>>>,
    tx: async_channel::Sender<SniEvent>,
}

#[interface(name = "org.kde.StatusNotifierWatcher")]
impl SniWatcher {
    async fn register_status_notifier_item(
        &mut self,
        service: &str,
        #[zbus(header)] header: zbus::message::Header<'_>,
    ) -> zbus::fdo::Result<()> {
        let id = if service.starts_with('/') {
            let sender = header.sender()
                .map(|s| s.as_str().to_string())
                .unwrap_or_default();
            format!("{}{}", sender, service)
        } else {
            service.to_string()
        };

        {
            let mut items = self.items.lock().unwrap();
            if !items.contains(&id) {
                items.push(id.clone());
            }
        }

        let _ = self.tx.send(SniEvent::ItemRegistered(id)).await;
        Ok(())
    }

    async fn register_status_notifier_host(
        &mut self,
        _service: &str,
    ) -> zbus::fdo::Result<()> {
        Ok(())
    }

    #[zbus(property)]
    fn registered_status_notifier_items(&self) -> Vec<String> {
        self.items.lock().unwrap().clone()
    }

    #[zbus(property)]
    fn is_status_notifier_host_registered(&self) -> bool {
        true
    }

    #[zbus(property)]
    fn protocol_version(&self) -> i32 {
        0
    }
}

pub fn start_watcher(tx: async_channel::Sender<SniEvent>) {
    crate::RUNTIME.get().unwrap().spawn(async move {
        let items = Arc::new(Mutex::new(Vec::<String>::new()));
        let watcher = SniWatcher {
            items: items.clone(),
            tx: tx.clone(),
        };

        let conn_result = zbus::connection::Builder::session()
            .and_then(|b| b.name("org.kde.StatusNotifierWatcher"))
            .and_then(|b| b.serve_at("/StatusNotifierWatcher", watcher));

        let conn = match conn_result {
            Ok(builder) => match builder.build().await {
                Ok(c) => c,
                Err(e) => {
                    log::warn!("SNI watcher failed to build connection: {}", e);
                    return;
                }
            },
            Err(e) => {
                log::warn!("SNI watcher builder failed: {}", e);
                return;
            }
        };

        log::info!("SNI watcher started on D-Bus");

        // Watch for name owner changes to handle disconnected items
        let _conn_ref = conn;
        let tx2 = tx.clone();
        let items2 = items.clone();

        // Simple polling to detect disconnected items
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            // For each registered item, check if it's still alive
            let item_list: Vec<String> = items2.lock().unwrap().clone();
            for item in &item_list {
                let (bus, _path) = parse_sni_id(item);
                // Try a simple D-Bus ping - if fails, remove the item
                // We'll skip this for now and just keep items until reconnect
                let _ = &bus;
            }
            let _ = tx2.clone();
        }
    });
}

fn parse_sni_id(id: &str) -> (String, String) {
    if let Some(idx) = id.find('/') {
        (id[..idx].to_string(), id[idx..].to_string())
    } else {
        (id.to_string(), "/StatusNotifierItem".to_string())
    }
}
