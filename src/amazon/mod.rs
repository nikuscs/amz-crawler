//! Amazon-specific modules for HTTP client, parsing, and data models.

pub mod client;
pub mod models;
pub mod parser;
pub mod regions;
pub mod selectors;

pub use client::{AmazonClient, AmazonSearch};
pub use models::{Price, PriceRange, Product, Rating};
pub use parser::Parser;
pub use regions::Region;
