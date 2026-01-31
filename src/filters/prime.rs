//! Prime-only filter.

use super::Filter;
use crate::amazon::Product;

/// Filters to only include Prime-eligible products.
pub struct PrimeFilter;

impl PrimeFilter {
    /// Creates a new Prime filter.
    pub fn new() -> Self {
        Self
    }
}

impl Default for PrimeFilter {
    fn default() -> Self {
        Self::new()
    }
}

impl Filter for PrimeFilter {
    fn matches(&self, product: &Product) -> bool {
        product.is_prime
    }

    fn description(&self) -> String {
        "Prime only".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_product(is_prime: bool) -> Product {
        Product {
            asin: "TEST".to_string(),
            title: "Test".to_string(),
            url: "https://amazon.com/dp/TEST".to_string(),
            image_url: None,
            price: None,
            rating: None,
            is_sponsored: false,
            is_prime,
            is_amazon_choice: false,
            in_stock: true,
            brand: None,
        }
    }

    #[test]
    fn test_prime_filter() {
        let filter = PrimeFilter::new();

        assert!(filter.matches(&make_product(true)));
        assert!(!filter.matches(&make_product(false)));
    }

    #[test]
    fn test_prime_filter_default() {
        let filter: PrimeFilter = Default::default();
        assert!(filter.matches(&make_product(true)));
        assert!(!filter.matches(&make_product(false)));
    }

    #[test]
    fn test_prime_filter_description() {
        let filter = PrimeFilter::new();
        assert_eq!(filter.description(), "Prime only");
    }
}
