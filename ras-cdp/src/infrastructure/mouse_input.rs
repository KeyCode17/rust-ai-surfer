use std::time::Instant;

use chromiumoxide::Page;
use chromiumoxide::cdp::browser_protocol::input::{
    DispatchMouseEventParams, DispatchMouseEventType, MouseButton,
};
use ras_errors::AppError;
use tokio::time::{Duration as TDur, sleep};

pub(crate) async fn dispatch_mouse_move(
    page: &Page,
    x: i32,
    y: i32,
    buttons_mask: i64,
) -> Result<(), AppError> {
    let mut b = DispatchMouseEventParams::builder()
        .r#type(DispatchMouseEventType::MouseMoved)
        .x(f64::from(x))
        .y(f64::from(y))
        .buttons(buttons_mask);
    if buttons_mask & 1 != 0 {
        b = b.button(MouseButton::Left);
    }
    let mv = b
        .build()
        .map_err(|e| AppError::ActionFailed(format!("mouse_move params: {e}")))?;
    page.execute(mv)
        .await
        .map_err(|e| AppError::ActionFailed(format!("mouse_move dispatch: {e}")))?;
    Ok(())
}

pub(crate) async fn dispatch_mouse_press(page: &Page, x: i32, y: i32) -> Result<(), AppError> {
    let move_params = DispatchMouseEventParams::builder()
        .r#type(DispatchMouseEventType::MouseMoved)
        .x(f64::from(x))
        .y(f64::from(y))
        .build()
        .map_err(|e| AppError::ActionFailed(format!("mouse_move params: {e}")))?;
    page.execute(move_params)
        .await
        .map_err(|e| AppError::ActionFailed(format!("mouse_move dispatch: {e}")))?;
    let press = DispatchMouseEventParams::builder()
        .r#type(DispatchMouseEventType::MousePressed)
        .x(f64::from(x))
        .y(f64::from(y))
        .button(MouseButton::Left)
        .click_count(1)
        .build()
        .map_err(|e| AppError::ActionFailed(format!("mouse_press params: {e}")))?;
    page.execute(press)
        .await
        .map_err(|e| AppError::ActionFailed(format!("mouse_press dispatch: {e}")))?;
    Ok(())
}

pub(crate) async fn dispatch_mouse_release(page: &Page, x: i32, y: i32) -> Result<(), AppError> {
    let release = DispatchMouseEventParams::builder()
        .r#type(DispatchMouseEventType::MouseReleased)
        .x(f64::from(x))
        .y(f64::from(y))
        .button(MouseButton::Left)
        .click_count(1)
        .build()
        .map_err(|e| AppError::ActionFailed(format!("mouse_release params: {e}")))?;
    page.execute(release)
        .await
        .map_err(|e| AppError::ActionFailed(format!("mouse_release dispatch: {e}")))?;
    Ok(())
}

/// Press at (x, y), hold for total_ms while emitting small jittered MouseMoved
/// events with `buttons=1`, then release. Approaches the target with intermediate
/// moves first so the input trace looks human, not teleported. Used to defeat
/// PerimeterX / HUMAN "Press & Hold" challenges that score on movement entropy.
pub(crate) async fn dispatch_mouse_hold(
    page: &Page,
    x: i32,
    y: i32,
    total_ms: u64,
) -> Result<(), AppError> {
    let steps = 6_i32;
    let start_x = x - 40;
    let start_y = y - 30;
    for i in 1..=steps {
        let t = f64::from(i) / f64::from(steps);
        let cx = (f64::from(start_x) + (f64::from(x - start_x)) * t).round() as i32;
        let cy = (f64::from(start_y) + (f64::from(y - start_y)) * t).round() as i32;
        let mv = DispatchMouseEventParams::builder()
            .r#type(DispatchMouseEventType::MouseMoved)
            .x(f64::from(cx))
            .y(f64::from(cy))
            .build()
            .map_err(|e| AppError::ActionFailed(format!("mouse_move params: {e}")))?;
        page.execute(mv)
            .await
            .map_err(|e| AppError::ActionFailed(format!("mouse_move dispatch: {e}")))?;
        sleep(TDur::from_millis(25)).await;
    }
    let press = DispatchMouseEventParams::builder()
        .r#type(DispatchMouseEventType::MousePressed)
        .x(f64::from(x))
        .y(f64::from(y))
        .button(MouseButton::Left)
        .buttons(1_i64)
        .click_count(1)
        .build()
        .map_err(|e| AppError::ActionFailed(format!("mouse_press params: {e}")))?;
    page.execute(press)
        .await
        .map_err(|e| AppError::ActionFailed(format!("mouse_press dispatch: {e}")))?;
    let started = Instant::now();
    let tick = TDur::from_millis(80);
    let total = TDur::from_millis(total_ms);
    let mut n: i64 = 0;
    while started.elapsed() < total {
        sleep(tick).await;
        n = n.wrapping_add(1);
        let dx = (((n.wrapping_mul(73)) % 3) - 1) as i32;
        let dy = (((n.wrapping_mul(31)) % 3) - 1) as i32;
        let mv = DispatchMouseEventParams::builder()
            .r#type(DispatchMouseEventType::MouseMoved)
            .x(f64::from(x + dx))
            .y(f64::from(y + dy))
            .button(MouseButton::Left)
            .buttons(1_i64)
            .build()
            .map_err(|e| AppError::ActionFailed(format!("hold_jitter params: {e}")))?;
        if let Err(e) = page.execute(mv).await {
            tracing::warn!(error = %e, "hold jitter dispatch failed; continuing");
        }
    }
    let release = DispatchMouseEventParams::builder()
        .r#type(DispatchMouseEventType::MouseReleased)
        .x(f64::from(x))
        .y(f64::from(y))
        .button(MouseButton::Left)
        .click_count(1)
        .build()
        .map_err(|e| AppError::ActionFailed(format!("mouse_release params: {e}")))?;
    page.execute(release)
        .await
        .map_err(|e| AppError::ActionFailed(format!("mouse_release dispatch: {e}")))?;
    Ok(())
}
