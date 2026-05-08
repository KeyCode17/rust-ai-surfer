pub mod application;
pub mod domain;
pub mod infrastructure;

pub use infrastructure::http::chat_openai_compatible::{ChatOpenAICompatible, OpenAiAuth};
