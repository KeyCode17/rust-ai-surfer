use std::sync::Arc;

use ras_errors::AppError;

use crate::domain::registry::ActionRegistry;
use crate::infrastructure::builtin::click::{ClickCoordinateAction, ClickElementAction};
use crate::infrastructure::builtin::done::DoneAction;
use crate::infrastructure::builtin::navigate::NavigateAction;
use crate::infrastructure::builtin::press_and_hold::{
    PressAndHoldCoordinateAction, PressAndHoldElementAction,
};
use crate::infrastructure::builtin::screenshot::ScreenshotAction;
use crate::infrastructure::builtin::scroll::ScrollAction;
use crate::infrastructure::builtin::select_option::SelectOptionAction;
use crate::infrastructure::builtin::type_text::TypeTextAction;
use crate::infrastructure::builtin::wait::WaitAction;

pub fn register_default_actions(registry: &mut ActionRegistry) -> Result<(), AppError> {
    registry.register(Arc::new(NavigateAction))?;
    registry.register(Arc::new(ClickElementAction))?;
    registry.register(Arc::new(ClickCoordinateAction))?;
    registry.register(Arc::new(PressAndHoldElementAction))?;
    registry.register(Arc::new(PressAndHoldCoordinateAction))?;
    registry.register(Arc::new(TypeTextAction))?;
    registry.register(Arc::new(SelectOptionAction))?;
    registry.register(Arc::new(ScrollAction))?;
    registry.register(Arc::new(ScreenshotAction))?;
    registry.register(Arc::new(WaitAction))?;
    registry.register(Arc::new(DoneAction))?;
    Ok(())
}
