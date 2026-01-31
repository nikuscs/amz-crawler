//! Data models for Amazon products, prices, and ratings.

use serde::{Deserialize, Serialize};

/// Represents an Amazon product with all available metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Product {
    /// Amazon Standard Identification Number
    pub asin: String,
    /// Product title
    pub title: String,
    /// Full product URL
    pub url: String,
    /// Product image URL
    pub image_url: Option<String>,
    /// Current price information
    pub price: Option<Price>,
    /// Rating and review information
    pub rating: Option<Rating>,
    /// Whether this is a sponsored listing
    pub is_sponsored: bool,
    /// Whether this has Prime shipping
    pub is_prime: bool,
    /// Whether this has the "Amazon's Choice" badge
    pub is_amazon_choice: bool,
    /// Whether the product is currently in stock
    pub in_stock: bool,
    /// Product brand if available
    pub brand: Option<String>,
}

impl Product {
    /// Returns the current price as f64 if available.
    pub fn current_price(&self) -> Option<f64> {
        self.price.as_ref().and_then(|p| if p.is_hidden { None } else { Some(p.current) })
    }

    /// Returns the star rating if available.
    pub fn stars(&self) -> Option<f32> {
        self.rating.as_ref().map(|r| r.stars)
    }

    /// Returns discount percentage if on sale.
    pub fn discount_percent(&self) -> Option<u8> {
        self.price.as_ref().and_then(|p| {
            p.original.map(|orig| {
                let discount = ((orig - p.current) / orig * 100.0).round() as u8;
                discount.min(99)
            })
        })
    }
}

/// Price information including current, original, and range prices.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Price {
    /// Current/sale price
    pub current: f64,
    /// Original price before discount (if on sale)
    pub original: Option<f64>,
    /// Currency code (USD, EUR, etc.)
    pub currency: String,
    /// Price range for variable-priced items
    pub range: Option<PriceRange>,
    /// True if price is "See price in cart"
    pub is_hidden: bool,
}

impl Price {
    /// Creates a simple price with just current value.
    pub fn simple(current: f64, currency: impl Into<String>) -> Self {
        Self { current, original: None, currency: currency.into(), range: None, is_hidden: false }
    }

    /// Creates a price with original/sale price.
    pub fn with_discount(current: f64, original: f64, currency: impl Into<String>) -> Self {
        Self {
            current,
            original: Some(original),
            currency: currency.into(),
            range: None,
            is_hidden: false,
        }
    }

    /// Creates a hidden price ("See price in cart").
    pub fn hidden(currency: impl Into<String>) -> Self {
        Self {
            current: 0.0,
            original: None,
            currency: currency.into(),
            range: None,
            is_hidden: true,
        }
    }

    /// Creates a price range.
    pub fn with_range(min: f64, max: Option<f64>, currency: impl Into<String>) -> Self {
        Self {
            current: min,
            original: None,
            currency: currency.into(),
            range: Some(PriceRange { min, max }),
            is_hidden: false,
        }
    }
}

/// Price range for items with variable pricing ("from $X" or "$X - $Y").
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceRange {
    /// Minimum price
    pub min: f64,
    /// Maximum price (None for "from $X" style)
    pub max: Option<f64>,
}

/// Product rating and review count.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rating {
    /// Star rating (0.0 - 5.0)
    pub stars: f32,
    /// Number of reviews
    pub review_count: u32,
}

impl Rating {
    /// Creates a new rating.
    pub fn new(stars: f32, review_count: u32) -> Self {
        Self { stars: stars.clamp(0.0, 5.0), review_count }
    }
}

/// Search results container with metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResults {
    /// Search query used
    pub query: String,
    /// Region searched
    pub region: String,
    /// Total results found (if available from page)
    pub total_results: Option<u32>,
    /// Products found
    pub products: Vec<Product>,
    /// Current page number
    pub page: u32,
    /// Whether there are more pages
    pub has_more: bool,
}

impl SearchResults {
    /// Creates new search results.
    pub fn new(query: impl Into<String>, region: impl Into<String>) -> Self {
        Self {
            query: query.into(),
            region: region.into(),
            total_results: None,
            products: Vec::new(),
            page: 1,
            has_more: false,
        }
    }

    /// Returns number of products.
    pub fn count(&self) -> usize {
        self.products.len()
    }

    /// Returns true if no products were found.
    pub fn is_empty(&self) -> bool {
        self.products.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_product() -> Product {
        Product {
            asin: "TEST123".to_string(),
            title: "Test Product".to_string(),
            url: "https://amazon.com/dp/TEST123".to_string(),
            image_url: None,
            price: Some(Price::with_discount(20.0, 40.0, "USD")),
            rating: Some(Rating::new(4.5, 100)),
            is_sponsored: false,
            is_prime: true,
            is_amazon_choice: false,
            in_stock: true,
            brand: Some("TestBrand".to_string()),
        }
    }

    #[test]
    fn test_price_simple() {
        let price = Price::simple(29.99, "USD");
        assert_eq!(price.current, 29.99);
        assert!(price.original.is_none());
        assert!(!price.is_hidden);
        assert!(price.range.is_none());
        assert_eq!(price.currency, "USD");
    }

    #[test]
    fn test_price_with_discount() {
        let price = Price::with_discount(19.99, 29.99, "USD");
        assert_eq!(price.current, 19.99);
        assert_eq!(price.original, Some(29.99));
        assert!(!price.is_hidden);
    }

    #[test]
    fn test_price_hidden() {
        let price = Price::hidden("USD");
        assert!(price.is_hidden);
        assert_eq!(price.current, 0.0);
        assert_eq!(price.currency, "USD");
    }

    #[test]
    fn test_price_with_range() {
        let price = Price::with_range(10.0, Some(20.0), "EUR");
        assert_eq!(price.current, 10.0);
        assert!(price.range.is_some());
        let range = price.range.unwrap();
        assert_eq!(range.min, 10.0);
        assert_eq!(range.max, Some(20.0));
    }

    #[test]
    fn test_price_range_no_max() {
        let price = Price::with_range(15.0, None, "GBP");
        let range = price.range.unwrap();
        assert_eq!(range.min, 15.0);
        assert!(range.max.is_none());
    }

    #[test]
    fn test_product_current_price() {
        let product = make_test_product();
        assert_eq!(product.current_price(), Some(20.0));

        // No price
        let mut product = make_test_product();
        product.price = None;
        assert!(product.current_price().is_none());

        // Hidden price
        let mut product = make_test_product();
        product.price = Some(Price::hidden("USD"));
        assert!(product.current_price().is_none());
    }

    #[test]
    fn test_product_stars() {
        let product = make_test_product();
        assert_eq!(product.stars(), Some(4.5));

        let mut product = make_test_product();
        product.rating = None;
        assert!(product.stars().is_none());
    }

    #[test]
    fn test_discount_percent() {
        let product = make_test_product();
        assert_eq!(product.discount_percent(), Some(50));

        // No original price
        let mut product = make_test_product();
        product.price = Some(Price::simple(20.0, "USD"));
        assert!(product.discount_percent().is_none());

        // No price at all
        let mut product = make_test_product();
        product.price = None;
        assert!(product.discount_percent().is_none());
    }

    #[test]
    fn test_discount_percent_clamping() {
        // 99% discount should cap at 99
        let mut product = make_test_product();
        product.price = Some(Price::with_discount(1.0, 1000.0, "USD"));
        assert_eq!(product.discount_percent(), Some(99));
    }

    #[test]
    fn test_rating_clamping() {
        let rating = Rating::new(6.0, 10);
        assert_eq!(rating.stars, 5.0);

        let rating = Rating::new(-1.0, 10);
        assert_eq!(rating.stars, 0.0);

        let rating = Rating::new(3.5, 50);
        assert_eq!(rating.stars, 3.5);
        assert_eq!(rating.review_count, 50);
    }

    #[test]
    fn test_search_results() {
        let mut results = SearchResults::new("test query", "us");
        assert_eq!(results.query, "test query");
        assert_eq!(results.region, "us");
        assert_eq!(results.page, 1);
        assert!(!results.has_more);
        assert!(results.is_empty());
        assert_eq!(results.count(), 0);

        results.products.push(make_test_product());
        assert!(!results.is_empty());
        assert_eq!(results.count(), 1);
    }

    #[test]
    fn test_product_serde() {
        let product = make_test_product();
        let json = serde_json::to_string(&product).unwrap();
        assert!(json.contains("TEST123"));
        assert!(json.contains("Test Product"));

        let parsed: Product = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.asin, product.asin);
        assert_eq!(parsed.title, product.title);
    }

    #[test]
    fn test_price_serde() {
        let price = Price::with_discount(19.99, 29.99, "USD");
        let json = serde_json::to_string(&price).unwrap();
        let parsed: Price = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.current, 19.99);
        assert_eq!(parsed.original, Some(29.99));
    }

    #[test]
    fn test_rating_serde() {
        let rating = Rating::new(4.5, 1000);
        let json = serde_json::to_string(&rating).unwrap();
        let parsed: Rating = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.stars, 4.5);
        assert_eq!(parsed.review_count, 1000);
    }
}
