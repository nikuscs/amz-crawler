//! Product filtering system with composable filters.

pub mod keyword;
pub mod price;
pub mod prime;
pub mod rating;

use crate::amazon::Product;

pub use keyword::KeywordFilter;
pub use price::PriceFilter;
pub use prime::PrimeFilter;
pub use rating::RatingFilter;

/// Trait for filtering products.
pub trait Filter: Send + Sync {
    /// Returns true if the product passes the filter.
    fn matches(&self, product: &Product) -> bool;

    /// Returns a description of this filter.
    fn description(&self) -> String;
}

/// A chain of filters that must all pass.
pub struct FilterChain {
    filters: Vec<Box<dyn Filter>>,
}

impl FilterChain {
    /// Creates an empty filter chain.
    pub fn new() -> Self {
        Self { filters: Vec::new() }
    }

    /// Adds a filter to the chain.
    pub fn add(&mut self, filter: impl Filter + 'static) -> &mut Self {
        self.filters.push(Box::new(filter));
        self
    }

    /// Checks if a product passes all filters.
    pub fn matches(&self, product: &Product) -> bool {
        self.filters.iter().all(|f| f.matches(product))
    }

    /// Filters a collection of products.
    pub fn apply(&self, products: Vec<Product>) -> Vec<Product> {
        products.into_iter().filter(|p| self.matches(p)).collect()
    }

    /// Returns true if no filters are configured.
    pub fn is_empty(&self) -> bool {
        self.filters.is_empty()
    }

    /// Returns the number of filters.
    pub fn len(&self) -> usize {
        self.filters.len()
    }

    /// Returns descriptions of all filters.
    pub fn descriptions(&self) -> Vec<String> {
        self.filters.iter().map(|f| f.description()).collect()
    }
}

impl Default for FilterChain {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for constructing a FilterChain from configuration.
pub struct FilterChainBuilder {
    chain: FilterChain,
}

impl FilterChainBuilder {
    /// Creates a new builder.
    pub fn new() -> Self {
        Self { chain: FilterChain::new() }
    }

    /// Adds a price range filter.
    pub fn price_range(mut self, min: Option<f64>, max: Option<f64>) -> Self {
        if min.is_some() || max.is_some() {
            self.chain.add(PriceFilter::new(min, max));
        }
        self
    }

    /// Adds a minimum rating filter.
    pub fn min_rating(mut self, min: Option<f32>) -> Self {
        if let Some(min) = min {
            self.chain.add(RatingFilter::new(min));
        }
        self
    }

    /// Adds a Prime-only filter.
    pub fn prime_only(mut self, enabled: bool) -> Self {
        if enabled {
            self.chain.add(PrimeFilter::new());
        }
        self
    }

    /// Adds a sponsored filter (excludes sponsored).
    pub fn no_sponsored(mut self, enabled: bool) -> Self {
        if enabled {
            self.chain.add(SponsoredFilter::new());
        }
        self
    }

    /// Adds required keywords filter.
    pub fn keywords(mut self, keywords: Vec<String>) -> Self {
        if !keywords.is_empty() {
            self.chain.add(KeywordFilter::required(keywords));
        }
        self
    }

    /// Adds excluded keywords filter.
    pub fn exclude_keywords(mut self, keywords: Vec<String>) -> Self {
        if !keywords.is_empty() {
            self.chain.add(KeywordFilter::excluded(keywords));
        }
        self
    }

    /// Builds the filter chain.
    pub fn build(self) -> FilterChain {
        self.chain
    }
}

impl Default for FilterChainBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Filter that excludes sponsored products.
pub struct SponsoredFilter;

impl SponsoredFilter {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SponsoredFilter {
    fn default() -> Self {
        Self::new()
    }
}

impl Filter for SponsoredFilter {
    fn matches(&self, product: &Product) -> bool {
        !product.is_sponsored
    }

    fn description(&self) -> String {
        "Exclude sponsored".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::amazon::models::{Price, Rating};

    fn make_product(price: f64, rating: f32, is_prime: bool, is_sponsored: bool) -> Product {
        Product {
            asin: "TEST".to_string(),
            title: "Test Product".to_string(),
            url: "https://amazon.com/dp/TEST".to_string(),
            image_url: None,
            price: Some(Price::simple(price, "USD")),
            rating: Some(Rating::new(rating, 100)),
            is_sponsored,
            is_prime,
            is_amazon_choice: false,
            in_stock: true,
            brand: None,
        }
    }

    fn make_product_with_title(title: &str, is_prime: bool, is_sponsored: bool) -> Product {
        Product {
            asin: "TEST".to_string(),
            title: title.to_string(),
            url: "https://amazon.com/dp/TEST".to_string(),
            image_url: None,
            price: Some(Price::simple(25.0, "USD")),
            rating: Some(Rating::new(4.0, 100)),
            is_sponsored,
            is_prime,
            is_amazon_choice: false,
            in_stock: true,
            brand: None,
        }
    }

    // FilterChain tests

    #[test]
    fn test_filter_chain_new() {
        let chain = FilterChain::new();
        assert!(chain.is_empty());
        assert_eq!(chain.len(), 0);
    }

    #[test]
    fn test_filter_chain_default() {
        let chain = FilterChain::default();
        assert!(chain.is_empty());
    }

    #[test]
    fn test_filter_chain() {
        let mut chain = FilterChain::new();
        chain.add(PriceFilter::new(Some(10.0), Some(50.0)));
        chain.add(RatingFilter::new(4.0));

        assert_eq!(chain.len(), 2);
        assert!(!chain.is_empty());

        // Should pass: price 25, rating 4.5
        let product = make_product(25.0, 4.5, true, false);
        assert!(chain.matches(&product));

        // Should fail: price too low
        let product = make_product(5.0, 4.5, true, false);
        assert!(!chain.matches(&product));

        // Should fail: rating too low
        let product = make_product(25.0, 3.5, true, false);
        assert!(!chain.matches(&product));
    }

    #[test]
    fn test_filter_chain_empty_matches_all() {
        let chain = FilterChain::new();
        let product = make_product(25.0, 4.5, true, true);
        assert!(chain.matches(&product));
    }

    #[test]
    fn test_filter_chain_apply() {
        let mut chain = FilterChain::new();
        chain.add(PriceFilter::new(Some(20.0), None));

        let products = vec![
            make_product(10.0, 4.0, true, false),
            make_product(30.0, 4.0, true, false),
            make_product(50.0, 4.0, true, false),
        ];

        let filtered = chain.apply(products);
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_filter_chain_descriptions() {
        let mut chain = FilterChain::new();
        chain.add(PriceFilter::range(10.0, 50.0));
        chain.add(RatingFilter::new(4.0));
        chain.add(PrimeFilter::new());

        let descriptions = chain.descriptions();
        assert_eq!(descriptions.len(), 3);
        assert!(descriptions[0].contains("Price"));
        assert!(descriptions[1].contains("Rating"));
        assert!(descriptions[2].contains("Prime"));
    }

    // FilterChainBuilder tests

    #[test]
    fn test_filter_chain_builder() {
        let chain = FilterChainBuilder::new()
            .price_range(Some(10.0), Some(100.0))
            .min_rating(Some(4.0))
            .prime_only(true)
            .no_sponsored(true)
            .build();

        assert_eq!(chain.len(), 4);
    }

    #[test]
    fn test_filter_chain_builder_default() {
        let builder = FilterChainBuilder::default();
        let chain = builder.build();
        assert!(chain.is_empty());
    }

    #[test]
    fn test_filter_chain_builder_no_filters_when_disabled() {
        let chain = FilterChainBuilder::new()
            .price_range(None, None)
            .min_rating(None)
            .prime_only(false)
            .no_sponsored(false)
            .keywords(Vec::new())
            .exclude_keywords(Vec::new())
            .build();

        assert!(chain.is_empty());
    }

    #[test]
    fn test_filter_chain_builder_keywords() {
        let chain = FilterChainBuilder::new()
            .keywords(vec!["gaming".to_string(), "mouse".to_string()])
            .exclude_keywords(vec!["refurbished".to_string()])
            .build();

        assert_eq!(chain.len(), 2);

        let product = make_product_with_title("Gaming Wireless Mouse", true, false);
        assert!(chain.matches(&product));

        let product = make_product_with_title("Refurbished Gaming Mouse", true, false);
        assert!(!chain.matches(&product));

        let product = make_product_with_title("Gaming Keyboard", true, false);
        assert!(!chain.matches(&product)); // Missing "mouse"
    }

    #[test]
    fn test_filter_chain_builder_price_min_only() {
        let chain = FilterChainBuilder::new().price_range(Some(20.0), None).build();

        assert_eq!(chain.len(), 1);

        let product = make_product(25.0, 4.0, true, false);
        assert!(chain.matches(&product));

        let product = make_product(10.0, 4.0, true, false);
        assert!(!chain.matches(&product));
    }

    #[test]
    fn test_filter_chain_builder_price_max_only() {
        let chain = FilterChainBuilder::new().price_range(None, Some(50.0)).build();

        assert_eq!(chain.len(), 1);

        let product = make_product(25.0, 4.0, true, false);
        assert!(chain.matches(&product));

        let product = make_product(100.0, 4.0, true, false);
        assert!(!chain.matches(&product));
    }

    // SponsoredFilter tests

    #[test]
    fn test_sponsored_filter() {
        let filter = SponsoredFilter::new();

        let product = make_product(25.0, 4.0, true, false);
        assert!(filter.matches(&product));

        let product = make_product(25.0, 4.0, true, true);
        assert!(!filter.matches(&product));
    }

    #[test]
    fn test_sponsored_filter_default() {
        let filter: SponsoredFilter = Default::default();
        let product = make_product(25.0, 4.0, true, false);
        assert!(filter.matches(&product));
    }

    #[test]
    fn test_sponsored_filter_description() {
        let filter = SponsoredFilter::new();
        assert_eq!(filter.description(), "Exclude sponsored");
    }

    // Integration test with all filters

    #[test]
    fn test_all_filters_combined() {
        let chain = FilterChainBuilder::new()
            .price_range(Some(20.0), Some(100.0))
            .min_rating(Some(4.0))
            .prime_only(true)
            .no_sponsored(true)
            .keywords(vec!["gaming".to_string()])
            .exclude_keywords(vec!["refurbished".to_string()])
            .build();

        assert_eq!(chain.len(), 6);

        // Product that passes all filters
        let mut product = make_product(50.0, 4.5, true, false);
        product.title = "Gaming Laptop".to_string();
        assert!(chain.matches(&product));

        // Fails price filter
        product = make_product(10.0, 4.5, true, false);
        product.title = "Gaming Laptop".to_string();
        assert!(!chain.matches(&product));

        // Fails rating filter
        product = make_product(50.0, 3.5, true, false);
        product.title = "Gaming Laptop".to_string();
        assert!(!chain.matches(&product));

        // Fails prime filter
        product = make_product(50.0, 4.5, false, false);
        product.title = "Gaming Laptop".to_string();
        assert!(!chain.matches(&product));

        // Fails sponsored filter
        product = make_product(50.0, 4.5, true, true);
        product.title = "Gaming Laptop".to_string();
        assert!(!chain.matches(&product));

        // Fails keyword filter
        product = make_product(50.0, 4.5, true, false);
        product.title = "Business Laptop".to_string();
        assert!(!chain.matches(&product));

        // Fails exclude filter
        product = make_product(50.0, 4.5, true, false);
        product.title = "Refurbished Gaming Laptop".to_string();
        assert!(!chain.matches(&product));
    }
}
