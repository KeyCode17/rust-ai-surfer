use chromiumoxide::Page;
use chromiumoxide::cdp::browser_protocol::dom::{
    BackendNodeId as CdpBackendNodeId, ResolveNodeParams,
};
use chromiumoxide::cdp::browser_protocol::input::{DispatchKeyEventParams, DispatchKeyEventType};
use chromiumoxide::cdp::js_protocol::runtime::CallFunctionOnParams;
use ras_errors::AppError;
use ras_types::BackendNodeId;

pub(crate) async fn click_backend_node(page: &Page, node: BackendNodeId) -> Result<(), AppError> {
    let resolve = ResolveNodeParams::builder()
        .backend_node_id(CdpBackendNodeId::new(node.0))
        .build();
    let resolved = page
        .execute(resolve)
        .await
        .map_err(|e| AppError::ActionFailed(format!("resolve_node: {e}")))?;
    let object_id = resolved
        .result
        .object
        .object_id
        .clone()
        .ok_or_else(|| AppError::ActionFailed("resolve_node: no object_id".into()))?;
    let call = CallFunctionOnParams::builder()
        .object_id(object_id)
        .function_declaration("function() { this.click(); }".to_string())
        .build()
        .map_err(|e| AppError::ActionFailed(format!("call_fn params: {e}")))?;
    page.execute(call)
        .await
        .map_err(|e| AppError::ActionFailed(format!("click_node call: {e}")))?;
    Ok(())
}

pub(crate) async fn type_chars(page: &Page, text: &str) -> Result<(), AppError> {
    for ch in text.chars() {
        let params = DispatchKeyEventParams::builder()
            .r#type(DispatchKeyEventType::Char)
            .text(ch.to_string())
            .build()
            .map_err(|e| AppError::ActionFailed(format!("key params: {e}")))?;
        page.execute(params)
            .await
            .map_err(|e| AppError::ActionFailed(format!("type_text dispatch: {e}")))?;
    }
    Ok(())
}
