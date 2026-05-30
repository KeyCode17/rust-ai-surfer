//! Browser-provider abstraction for tenant session isolation.

use std::sync::Arc;

use async_trait::async_trait;
use ras_cdp::BrowserPort;
use ras_errors::AppError;
use ras_types::ContextId;

/// Supplies an isolated browser context and the port to drive it.
///
/// Swap implementations to choose the isolation strategy:
/// - `SharedBrowserProvider` — one Chromium process, one CDP context per tenant.
/// - Future: process-per-tenant via a pool of browser launchers.
#[async_trait]
pub trait BrowserProvider: Send + Sync {
    /// Allocate a fresh browser context and return the port + context id.
    async fn acquire(&self) -> Result<(Arc<dyn BrowserPort>, ContextId), AppError>;
    /// Release a context obtained via [`acquire`][Self::acquire].
    async fn release(&self, ctx: &ContextId) -> Result<(), AppError>;
}

/// One shared browser; each session gets its own CDP BrowserContext.
pub struct SharedBrowserProvider {
    browser: Arc<dyn BrowserPort>,
}

impl SharedBrowserProvider {
    /// Wrap an existing browser adapter.
    pub fn new(browser: Arc<dyn BrowserPort>) -> Self {
        Self { browser }
    }
}

#[async_trait]
impl BrowserProvider for SharedBrowserProvider {
    async fn acquire(&self) -> Result<(Arc<dyn BrowserPort>, ContextId), AppError> {
        let ctx = self.browser.create_context().await?;
        Ok((self.browser.clone(), ctx))
    }

    async fn release(&self, ctx: &ContextId) -> Result<(), AppError> {
        self.browser.close_context(ctx).await
    }
}
