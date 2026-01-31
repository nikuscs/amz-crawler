//! Minimum rating filter.

use super::Filter;
use crate::amazon::Product;

/// Filters products by minimum star rating.
pub struct RatingFilter {
    min_stars: f32,
}

impl RatingFilter {
    /// Creates a new rating filter with minimum stars.
    pub fn new(min_stars: f32) -> Self {
        Self { min_stars: min_stars.clamp(0.0, 5.0) }
    }
}

impl Filter for RatingFilter {
    fn matches(&self, product: &Product) -> bool {
        // Products without rating pass the filter (don't exclude them)
        let Some(stars) = product.stars() else {
            return true;
        };

        stars >= self.min_stars
    }

    fn description(&self) -> String {
        format!("Rating: >= {:.1} stars", self.min_stars)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::amazon::models::Rating;

    fn make_product(rating: Option<f32>) -> Product {
        Product {
            asin: "TEST".to_string(),
            title: "Test".to_string(),
            url: "https://amazon.com/dp/TEST".to_string(),
            image_url: None,
            price: None,
            rating: rating.map(|r| Rating::new(r, 100)),
            is_sponsored: false,
            is_prime: false,
            is_amazon_choice: false,
            in_stock: true,
            brand: None,
        }
    }

    #[test]
    fn test_rating_filter() {
        let filter = RatingFilter::new(4.0);

        assert!(!filter.matches(&make_product(Some(3.5))));
        assert!(filter.matches(&make_product(Some(4.0))));
        assert!(filter.matches(&make_product(Some(4.5))));
        assert!(filter.matches(&make_product(Some(5.0))));
    }

    #[test]
    fn test_no_rating_passes() {
        let filter = RatingFilter::new(4.0);
        assert!(filter.matches(&make_product(None)));
    }

    #[test]
    fn test_clamping() {
        let filter = RatingFilter::new(6.0);
        assert_eq!(filter.min_stars, 5.0);

        let filter = RatingFilter::new(-1.0);
        assert_eq!(filter.min_stars, 0.0);
    }

    #[test]
    fn test_description() {
        let filter = RatingFilter::new(4.0);
        assert_eq!(filter.description(), "Rating: >= 4.0 stars");

        let filter = RatingFilter::new(3.5);
        assert_eq!(filter.description(), "Rating: >= 3.5 stars");
    }

    #[test]
    fn test_exact_boundary() {
        let filter = RatingFilter::new(4.0);
        assert!(filter.matches(&make_product(Some(4.0))));
        assert!(!filter.matches(&make_product(Some(3.9))));
    }

    #[test]
    fn test_zero_rating() {
        let filter = RatingFilter::new(0.0);
        assert!(filter.matches(&make_product(Some(0.0))));
        assert!(filter.matches(&make_product(Some(1.0))));
        assert!(filter.matches(&make_product(Some(5.0))));
    }

    #[test]
    fn test_max_rating() {
        let filter = RatingFilter::new(5.0);
        assert!(filter.matches(&make_product(Some(5.0))));
        assert!(!filter.matches(&make_product(Some(4.9))));
    }
}
