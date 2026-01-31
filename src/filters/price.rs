//! Price range filter.

use super::Filter;
use crate::amazon::Product;

/// Filters products by price range.
pub struct PriceFilter {
    min: Option<f64>,
    max: Option<f64>,
}

impl PriceFilter {
    /// Creates a new price filter with optional min/max bounds.
    pub fn new(min: Option<f64>, max: Option<f64>) -> Self {
        Self { min, max }
    }

    /// Creates a filter with only minimum price.
    pub fn min(price: f64) -> Self {
        Self { min: Some(price), max: None }
    }

    /// Creates a filter with only maximum price.
    pub fn max(price: f64) -> Self {
        Self { min: None, max: Some(price) }
    }

    /// Creates a filter with both min and max.
    pub fn range(min: f64, max: f64) -> Self {
        Self { min: Some(min), max: Some(max) }
    }
}

impl Filter for PriceFilter {
    fn matches(&self, product: &Product) -> bool {
        // Products without price pass the filter (don't exclude them)
        let Some(price) = product.current_price() else {
            return true;
        };

        // Check minimum
        if let Some(min) = self.min {
            if price < min {
                return false;
            }
        }

        // Check maximum
        if let Some(max) = self.max {
            if price > max {
                return false;
            }
        }

        true
    }

    fn description(&self) -> String {
        match (self.min, self.max) {
            (Some(min), Some(max)) => format!("Price: ${:.2} - ${:.2}", min, max),
            (Some(min), None) => format!("Price: >= ${:.2}", min),
            (None, Some(max)) => format!("Price: <= ${:.2}", max),
            (None, None) => "Price: any".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::amazon::models::Price;

    fn make_product(price: Option<f64>) -> Product {
        Product {
            asin: "TEST".to_string(),
            title: "Test".to_string(),
            url: "https://amazon.com/dp/TEST".to_string(),
            image_url: None,
            price: price.map(|p| Price::simple(p, "USD")),
            rating: None,
            is_sponsored: false,
            is_prime: false,
            is_amazon_choice: false,
            in_stock: true,
            brand: None,
        }
    }

    fn make_hidden_price_product() -> Product {
        Product {
            asin: "TEST".to_string(),
            title: "Test".to_string(),
            url: "https://amazon.com/dp/TEST".to_string(),
            image_url: None,
            price: Some(Price::hidden("USD")),
            rating: None,
            is_sponsored: false,
            is_prime: false,
            is_amazon_choice: false,
            in_stock: true,
            brand: None,
        }
    }

    #[test]
    fn test_price_range() {
        let filter = PriceFilter::range(10.0, 50.0);

        assert!(!filter.matches(&make_product(Some(5.0))));
        assert!(filter.matches(&make_product(Some(10.0))));
        assert!(filter.matches(&make_product(Some(30.0))));
        assert!(filter.matches(&make_product(Some(50.0))));
        assert!(!filter.matches(&make_product(Some(55.0))));
    }

    #[test]
    fn test_no_price_passes() {
        let filter = PriceFilter::range(10.0, 50.0);
        assert!(filter.matches(&make_product(None)));
    }

    #[test]
    fn test_hidden_price_passes() {
        let filter = PriceFilter::range(10.0, 50.0);
        assert!(filter.matches(&make_hidden_price_product()));
    }

    #[test]
    fn test_min_only() {
        let filter = PriceFilter::min(20.0);
        assert!(!filter.matches(&make_product(Some(10.0))));
        assert!(filter.matches(&make_product(Some(20.0))));
        assert!(filter.matches(&make_product(Some(100.0))));
    }

    #[test]
    fn test_max_only() {
        let filter = PriceFilter::max(50.0);
        assert!(filter.matches(&make_product(Some(10.0))));
        assert!(filter.matches(&make_product(Some(50.0))));
        assert!(!filter.matches(&make_product(Some(100.0))));
    }

    #[test]
    fn test_new_with_options() {
        let filter = PriceFilter::new(Some(10.0), Some(50.0));
        assert!(filter.matches(&make_product(Some(30.0))));
        assert!(!filter.matches(&make_product(Some(5.0))));
        assert!(!filter.matches(&make_product(Some(55.0))));
    }

    #[test]
    fn test_new_no_bounds() {
        let filter = PriceFilter::new(None, None);
        assert!(filter.matches(&make_product(Some(0.01))));
        assert!(filter.matches(&make_product(Some(1000000.0))));
        assert!(filter.matches(&make_product(None)));
    }

    #[test]
    fn test_description_range() {
        let filter = PriceFilter::range(10.0, 50.0);
        assert_eq!(filter.description(), "Price: $10.00 - $50.00");
    }

    #[test]
    fn test_description_min_only() {
        let filter = PriceFilter::min(20.0);
        assert_eq!(filter.description(), "Price: >= $20.00");
    }

    #[test]
    fn test_description_max_only() {
        let filter = PriceFilter::max(50.0);
        assert_eq!(filter.description(), "Price: <= $50.00");
    }

    #[test]
    fn test_description_any() {
        let filter = PriceFilter::new(None, None);
        assert_eq!(filter.description(), "Price: any");
    }

    #[test]
    fn test_boundary_values() {
        let filter = PriceFilter::range(10.0, 50.0);

        // Exactly at boundaries
        assert!(filter.matches(&make_product(Some(10.0))));
        assert!(filter.matches(&make_product(Some(50.0))));

        // Just outside boundaries
        assert!(!filter.matches(&make_product(Some(9.99))));
        assert!(!filter.matches(&make_product(Some(50.01))));
    }
}
