//! Token Optimizer Library
//! 
//! A Rust library for optimizing token usage in AI coding agents through
//! intelligent caching, analysis, and ML-powered enhancements.

pub mod types;
pub mod utils;
pub mod analyzers;
pub mod cache;
pub mod generators;
pub mod ml;

#[cfg(test)]
pub mod integration_test;

#[cfg(test)]
pub mod e2e_calendar_test;

// Re-export commonly used types
pub use types::*;
pub use cache::{CacheManager, SmartCache};
pub use analyzers::FileAnalyzer;
pub use generators::{ProjectOverviewGenerator, ReportGenerator};
pub use ml::{MLConfig, MLCoordinator, PluginManager};

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Library name
pub const NAME: &str = env!("CARGO_PKG_NAME");

/// Get library information
pub fn info() -> String {
    format!("{} v{}", NAME, VERSION)
}