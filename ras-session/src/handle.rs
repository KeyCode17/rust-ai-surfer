//! Non-generic handle to a single live session.

use std::sync::Arc;
use std::sync::atomic::Ordering::SeqCst;

use ras_agent::AgentHistoryList;
use ras_agent::application::run_agent::RunAgent;
use ras_events::EventReceiver;

use crate::config::{AgentSessionId, SessionError};
use crate::entry::{SessionEntry, SessionRegistry};
use crate::provider::BrowserProvider;

/// Cloneable handle that can run tasks, observe events, and close a session.
///
/// Intentionally non-generic over `Owner`: removal goes through an
/// owner-erasing [`SessionRegistry`] trait object.
#[derive(Clone)]
pub struct SessionHandle {
    entry: Arc<SessionEntry>,
    provider: Arc<dyn BrowserProvider>,
    registry: Arc<dyn SessionRegistry>,
}

impl SessionHandle {
    pub(crate) fn new(
        entry: Arc<SessionEntry>,
        provider: Arc<dyn BrowserProvider>,
        registry: Arc<dyn SessionRegistry>,
    ) -> Self {
        Self {
            entry,
            provider,
            registry,
        }
    }

    /// This session's id.
    #[must_use]
    pub fn id(&self) -> AgentSessionId {
        self.entry.id.clone()
    }

    /// Subscribe to this session's browser-event stream.
    #[must_use]
    pub fn events(&self) -> EventReceiver {
        self.entry.bus.subscribe()
    }

    /// Run an agent task on this session.
    ///
    /// Returns [`SessionError::Busy`] if a task is already running.
    pub async fn run(&self, task: impl Into<String>) -> Result<AgentHistoryList, SessionError> {
        if self.entry.running.swap(true, SeqCst) {
            return Err(SessionError::Busy);
        }
        self.entry.touch();
        let result = self.execute(task.into()).await;
        self.entry.running.store(false, SeqCst);
        self.entry.touch();
        result.map_err(SessionError::Browser)
    }

    async fn execute(&self, task: String) -> Result<AgentHistoryList, ras_errors::AppError> {
        let p = &self.entry.params;
        let mut agent = RunAgent::new(
            task,
            p.llm.clone(),
            p.registry.clone(),
            self.entry.browser.clone(),
            self.entry.bus.clone(),
        )
        .with_target(self.entry.tab.clone())
        .with_max_steps(p.max_steps);
        if let Some(dom) = p.dom_extractor.clone() {
            agent = agent.with_dom_extractor(dom);
        }
        if let Some(sink) = p.screenshot_sink.clone() {
            agent = agent.with_screenshot_sink(sink);
        }
        agent.execute().await
    }

    /// Release the browser context and forget this session. Idempotent.
    pub async fn close(self) -> Result<(), SessionError> {
        self.provider.release(&self.entry.ctx).await?;
        self.registry.forget(&self.entry.id).await;
        Ok(())
    }
}
