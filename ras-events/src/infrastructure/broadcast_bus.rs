use async_broadcast::{Receiver, Sender, broadcast};
use async_trait::async_trait;
use ras_errors::AppError;
use tokio::sync::Mutex;

use crate::domain::browser_event::BrowserEvent;
use crate::domain::repository::EventBus;

pub struct BroadcastBus {
    tx: Mutex<Sender<BrowserEvent>>,
    rx: Receiver<BrowserEvent>,
}

impl BroadcastBus {
    #[must_use]
    pub fn new(capacity: usize) -> Self {
        let (mut tx, rx) = broadcast::<BrowserEvent>(capacity);
        tx.set_overflow(true);
        Self {
            tx: Mutex::new(tx),
            rx,
        }
    }
}

impl Default for BroadcastBus {
    fn default() -> Self {
        Self::new(256)
    }
}

#[async_trait]
impl EventBus for BroadcastBus {
    async fn publish(&self, event: BrowserEvent) -> Result<(), AppError> {
        let tx = self.tx.lock().await;
        tx.broadcast_direct(event)
            .await
            .map(|_| ())
            .map_err(|e| AppError::InternalError(format!("event publish: {e}")))
    }

    fn subscribe(&self) -> Receiver<BrowserEvent> {
        self.rx.new_receiver()
    }
}

impl std::fmt::Debug for BroadcastBus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BroadcastBus").finish()
    }
}
