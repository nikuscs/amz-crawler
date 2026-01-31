//! Integration tests for the HTML parser using fixture files.

use amz_crawler::amazon::parser::Parser;
use amz_crawler::amazon::regions::Region;

const SEARCH_FIXTURE: &str = include_str!("fixtures/search_result.html");

#[test]
fn test_parse_search_results() {
    let parser = Parser::new(Region::Us);
    let results = parser.parse_search(SEARCH_FIXTURE, "wireless mouse", 1).unwrap();

    // Should have parsed 2 products (third one has empty ASIN)
    assert_eq!(results.count(), 2);
    assert!(results.has_more);

    // Check first product
    let product = &results.products[0];
    assert_eq!(product.asin, "B08N5WRWNW");
    assert!(product.title.contains("Logitech"));
    assert!(product.is_prime);
    assert!(!product.is_sponsored);

    // Check price
    let price = product.price.as_ref().unwrap();
    assert_eq!(price.current, 99.99);
    assert_eq!(price.original, Some(129.99));
    assert_eq!(price.currency, "USD");

    // Check rating
    let rating = product.rating.as_ref().unwrap();
    assert_eq!(rating.stars, 4.7);
    assert_eq!(rating.review_count, 12345);

    // Check second product (sponsored)
    let product = &results.products[1];
    assert_eq!(product.asin, "B09HMZ6S1Y");
    assert!(product.is_sponsored);
    assert!(!product.is_prime);
}

#[test]
fn test_parse_empty_results() {
    let parser = Parser::new(Region::Us);
    let html = r#"
        <html>
        <body>
            <div class="s-no-search-results">No results found</div>
        </body>
        </html>
    "#;

    let results = parser.parse_search(html, "nonexistent", 1).unwrap();
    assert!(results.is_empty());
}

#[test]
fn test_filter_integration() {
    use amz_crawler::filters::FilterChainBuilder;

    let parser = Parser::new(Region::Us);
    let results = parser.parse_search(SEARCH_FIXTURE, "wireless mouse", 1).unwrap();

    // Build filter chain
    let filters = FilterChainBuilder::new()
        .price_range(Some(50.0), Some(150.0))
        .min_rating(Some(4.5))
        .no_sponsored(true)
        .build();

    let filtered = filters.apply(results.products);

    // Only the Logitech should pass (Razer is sponsored and under $50)
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].asin, "B08N5WRWNW");
}
