//! The hot-reload state.

use std::sync::Arc;

use dioxus::prelude::Template;
use tokio::sync::{
    watch::{channel, Receiver},
    RwLock,
};

/// The hot-reload state.
#[derive(Clone)]
pub struct HotReloadState {
    /// The cache of all templates that have been modified since the last time we checked
    pub templates: Arc<RwLock<std::collections::HashSet<dioxus::prelude::Template<'static>>>>,
    /// The channel to send messages to the hot reload thread
    pub message_receiver: Receiver<Option<Template<'static>>>,
}

impl Default for HotReloadState {
    fn default() -> Self {
        let templates = Arc::new(RwLock::new(std::collections::HashSet::new()));
        let (tx, rx) = channel(None);

        dioxus_hot_reload::connect({
            let templates = templates.clone();
            move |msg| match msg {
                dioxus_hot_reload::HotReloadMsg::UpdateTemplate(template) => {
                    {
                        let mut templates = templates.blocking_write();
                        templates.insert(template);
                    }

                    if let Err(err) = tx.send(Some(template)) {
                        tracing::error!("Failed to send hot reload message: {}", err);
                    }
                }
                dioxus_hot_reload::HotReloadMsg::Shutdown => {
                    std::process::exit(0);
                }
            }
        });

        Self {
            templates,
            message_receiver: rx,
        }
    }
}

// Hot reloading can be expensive to start so we spawn a new thread
static HOT_RELOAD_STATE: tokio::sync::OnceCell<HotReloadState> = tokio::sync::OnceCell::const_new();

/// Spawns hot_reload.
pub async fn spawn_hot_reload() -> &'static HotReloadState {
    HOT_RELOAD_STATE
        .get_or_init(|| async {
            println!("spinning up hot reloading");
            let r = tokio::task::spawn_blocking(HotReloadState::default)
                .await
                .unwrap();
            println!("hot reloading ready");
            r
        })
        .await
}
