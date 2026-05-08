use async_trait::async_trait;
use ras_errors::AppError;

use crate::domain::browser_event::BrowserEvent;

#[async_trait]
pub trait EventBus: Send + Sync + 'static {
    async fn publish(&self, event: BrowserEvent) -> Result<(), AppError>;
    fn subscribe(&self) -> EventReceiver;
}

pub type EventReceiver = async_broadcast::Receiver<BrowserEvent>;
