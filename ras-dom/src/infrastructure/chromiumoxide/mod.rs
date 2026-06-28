pub mod clickable_naming;
pub mod clickables;
pub mod extractor;
pub mod highlight;
pub mod snapshot;
pub mod snapshot_parser;
#[cfg(test)]
mod snapshot_parser_tests;

pub use extractor::ChromiumoxideDomExtractor;
