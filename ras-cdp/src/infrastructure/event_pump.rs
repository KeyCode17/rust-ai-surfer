//! CDP event forwarding: arms listeners on a [`chromiumoxide::Page`] and
//! publishes mapped [`BrowserEvent`] values to the supplied bus.

use std::sync::Arc;

use chromiumoxide::Page;
use chromiumoxide::cdp::browser_protocol::page::{
    DialogType, EventFrameNavigated, EventJavascriptDialogOpening,
};
use futures::StreamExt;
use ras_errors::AppError;
use ras_events::{BrowserEvent, DialogKind, EventBus};
use ras_types::TargetId;
use url::Url;

/// Arm CDP listeners on `page` and forward mapped events to `bus`.
///
/// Listeners are armed before returning; the spawned tasks run until the
/// page closes (streams end).
pub(crate) async fn attach(
    page: &Page,
    target: TargetId,
    bus: Arc<dyn EventBus>,
) -> Result<(), AppError> {
    let nav_stream = page
        .event_listener::<EventFrameNavigated>()
        .await
        .map_err(|e| AppError::ActionFailed(format!("listen frameNavigated: {e}")))?;

    let nav_target = target.clone();
    let nav_bus = bus.clone();
    tokio::spawn(async move {
        let mut s = nav_stream;
        while let Some(ev) = s.next().await {
            if ev.frame.parent_id.is_some() {
                continue;
            }
            if let Ok(url) = Url::parse(&ev.frame.url) {
                let _ = nav_bus
                    .publish(BrowserEvent::NavigationCompleted {
                        target: nav_target.clone(),
                        url,
                    })
                    .await;
            }
        }
    });

    let dialog_stream = page
        .event_listener::<EventJavascriptDialogOpening>()
        .await
        .map_err(|e| AppError::ActionFailed(format!("listen dialog: {e}")))?;

    tokio::spawn(async move {
        let mut s = dialog_stream;
        while let Some(ev) = s.next().await {
            let kind = map_dialog_kind(&ev.r#type);
            let _ = bus
                .publish(BrowserEvent::DialogOpened {
                    kind,
                    message: ev.message.clone(),
                })
                .await;
        }
    });

    Ok(())
}

fn map_dialog_kind(t: &DialogType) -> DialogKind {
    match t {
        DialogType::Alert => DialogKind::Alert,
        DialogType::Confirm => DialogKind::Confirm,
        DialogType::Prompt => DialogKind::Prompt,
        DialogType::Beforeunload => DialogKind::BeforeUnload,
    }
}
