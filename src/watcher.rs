use crate::config::AppConfig;
use notify_debouncer_mini::{new_debouncer, notify::RecursiveMode};
use std::path::Path;
use std::sync::{Arc, RwLock};
use std::time::Duration;
use tracing::{error, info};

pub fn start_watcher(path_str: String, config_lock: Arc<RwLock<AppConfig>>) {
    std::thread::spawn(move || {
        let path = Path::new(&path_str);
        let dir = path.parent().unwrap_or_else(|| Path::new("."));

        let (tx, rx) = std::sync::mpsc::channel();
        let mut debouncer = new_debouncer(Duration::from_secs(2), tx).unwrap();

        debouncer
            .watcher()
            .watch(dir, RecursiveMode::NonRecursive)
            .expect("Failed to start file watcher");

        for res in rx {
            match res {
                Ok(events) => {
                    for event in events {
                        if event.path == path {
                            info!("Config file modified, attempting live reload...");
                            match AppConfig::load(&path_str) {
                                Ok(new_config) => {
                                    let mut w = config_lock.write().unwrap();
                                    *w = new_config;
                                    info!("Configuration successfully reloaded in memory.");
                                }
                                Err(e) => {
                                    error!("Failed to parse new config, ignoring update: {}", e);
                                }
                            }
                        }
                    }
                }
                Err(e) => error!("Watcher error: {:?}", e),
            }
        }
    });
}