use std::io;

use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt;

pub fn init_logger() {
    init_logger_with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")));
}

pub fn init_logger_with(filter: EnvFilter) {
    fmt()
        .with_env_filter(filter)
        .with_writer(io::stderr)
        .with_target(false)
        .compact()
        .try_init()
        .ok();
}
