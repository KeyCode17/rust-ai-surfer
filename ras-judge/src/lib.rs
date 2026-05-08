pub mod domain;
pub mod application;
pub mod infrastructure;

pub use domain::judgement::{JudgementResult, Verdict};
pub use domain::repository::JudgePort;
