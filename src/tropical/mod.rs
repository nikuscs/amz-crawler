//! TropicalPrice integration for EU Amazon price comparison.
//!
//! This module provides functionality to compare prices across EU Amazon stores
//! using TropicalPrice.com as a data source.

mod client;
mod models;
mod parser;

pub use client::{TropicalClient, TropicalSearch};
pub use models::{CountryPrice, PriceComparison, TropicalProduct};
