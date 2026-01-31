//! amz-crawler - Fast, stateless Amazon product search CLI
//!
//! A Rust port of Python amzSear with TLS fingerprint emulation
//! for reliable scraping without detection.

pub mod amazon;
pub mod commands;
pub mod config;
pub mod filters;
pub mod format;

#[cfg(feature = "tropical")]
pub mod tropical;

pub use amazon::models::{Price, PriceRange, Product, Rating};
pub use amazon::regions::Region;
pub use config::Config;
