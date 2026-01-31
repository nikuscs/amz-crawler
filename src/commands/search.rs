//! Search command implementation.

use crate::amazon::{AmazonClient, AmazonSearch, Parser, Product};
use crate::config::Config;
use crate::filters::FilterChainBuilder;
use crate::format::Formatter;
use anyhow::{Context, Result};
use tracing::{debug, info};

/// Executes a product search.
pub struct SearchCommand {
    config: Config,
}

impl SearchCommand {
    /// Creates a new search command.
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// Executes the search and returns formatted output.
    pub async fn execute(&self, query: &str) -> Result<String> {
        let client =
            AmazonClient::new(&self.config).await.context("Failed to create HTTP client")?;

        self.execute_with_client(&client, query).await
    }

    /// Executes the search with a provided client (for testing).
    pub async fn execute_with_client(
        &self,
        client: &impl AmazonSearch,
        query: &str,
    ) -> Result<String> {
        info!("Searching for: {}", query);

        let parser = Parser::new(client.region());

        // Build filter chain
        let filters = FilterChainBuilder::new()
            .price_range(self.config.min_price, self.config.max_price)
            .min_rating(self.config.min_rating)
            .prime_only(self.config.prime_only)
            .no_sponsored(self.config.no_sponsored)
            .keywords(self.config.keywords.clone())
            .exclude_keywords(self.config.exclude_keywords.clone())
            .build();

        if !filters.is_empty() {
            debug!("Active filters: {}", filters.descriptions().join(", "));
        }

        let mut all_products: Vec<Product> = Vec::new();
        let mut page = 1;
        let max_pages = 10; // Safety limit

        // Fetch pages until we have enough results
        while all_products.len() < self.config.max_results && page <= max_pages {
            debug!("Fetching page {}", page);

            let html = client.search(query, page).await?;
            let results = parser.parse_search(&html, query, page)?;

            if results.is_empty() {
                debug!("No results on page {}, stopping", page);
                break;
            }

            // Apply filters
            let filtered = filters.apply(results.products);
            debug!(
                "Page {} returned {} products ({} after filtering)",
                page,
                results.total_results.unwrap_or(0),
                filtered.len()
            );

            all_products.extend(filtered);

            if !results.has_more {
                debug!("No more pages available");
                break;
            }

            page += 1;
        }

        // Truncate to max_results
        all_products.truncate(self.config.max_results);

        info!("Found {} products matching criteria", all_products.len());

        // Format output
        let formatter = Formatter::new(self.config.format);
        Ok(formatter.format_products(&all_products))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::amazon::Region;
    use crate::config::OutputFormat;
    use async_trait::async_trait;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;

    /// Mock Amazon client for testing.
    struct MockAmazonClient {
        search_responses: Vec<String>,
        product_responses: Vec<String>,
        search_call_count: Arc<AtomicU32>,
        region: Region,
    }

    impl MockAmazonClient {
        fn new(search_responses: Vec<String>) -> Self {
            Self {
                search_responses,
                product_responses: Vec::new(),
                search_call_count: Arc::new(AtomicU32::new(0)),
                region: Region::Us,
            }
        }

        fn call_count(&self) -> u32 {
            self.search_call_count.load(Ordering::SeqCst)
        }
    }

    #[async_trait]
    impl AmazonSearch for MockAmazonClient {
        async fn search(&self, _query: &str, page: u32) -> Result<String> {
            self.search_call_count.fetch_add(1, Ordering::SeqCst);
            let idx = (page - 1) as usize;
            if idx < self.search_responses.len() {
                Ok(self.search_responses[idx].clone())
            } else {
                Ok("<html></html>".to_string())
            }
        }

        async fn product(&self, _asin: &str) -> Result<String> {
            if !self.product_responses.is_empty() {
                Ok(self.product_responses[0].clone())
            } else {
                Ok("<html></html>".to_string())
            }
        }

        fn region(&self) -> Region {
            self.region
        }
    }

    fn make_test_config() -> Config {
        Config {
            region: Region::Us,
            proxy: None,
            delay_ms: 0,
            delay_jitter_ms: 0,
            max_results: 5,
            format: OutputFormat::Table,
            min_price: None,
            max_price: None,
            min_rating: None,
            prime_only: false,
            no_sponsored: false,
            keywords: Vec::new(),
            exclude_keywords: Vec::new(),
        }
    }

    fn make_search_html(products: &[(&str, &str, f64)]) -> String {
        let mut html = String::from("<html><body>");
        for (asin, title, price) in products {
            html.push_str(&format!(
                r#"<div data-component-type="s-search-result" data-asin="{}">
                    <h2><a class="a-link-normal" href="/dp/{}"><span>{}</span></a></h2>
                    <span class="a-price"><span class="a-offscreen">${:.2}</span></span>
                </div>"#,
                asin, asin, title, price
            ));
        }
        html.push_str("</body></html>");
        html
    }

    #[tokio::test]
    async fn test_search_command_basic() {
        let html =
            make_search_html(&[("B001", "Product One", 19.99), ("B002", "Product Two", 29.99)]);

        let client = MockAmazonClient::new(vec![html]);
        let config = make_test_config();
        let cmd = SearchCommand::new(config);

        let result = cmd.execute_with_client(&client, "test").await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.contains("B001"));
        assert!(output.contains("B002"));
        assert!(output.contains("Product One"));
    }

    #[tokio::test]
    async fn test_search_command_empty_results() {
        let client = MockAmazonClient::new(vec!["<html></html>".to_string()]);
        let config = make_test_config();
        let cmd = SearchCommand::new(config);

        let result = cmd.execute_with_client(&client, "nonexistent").await;
        assert!(result.is_ok());
        assert!(result.unwrap().contains("No products found"));
    }

    #[tokio::test]
    async fn test_search_command_with_filters() {
        let html = make_search_html(&[
            ("B001", "Cheap Product", 9.99),
            ("B002", "Mid Product", 25.00),
            ("B003", "Expensive Product", 100.00),
        ]);

        let client = MockAmazonClient::new(vec![html]);
        let mut config = make_test_config();
        config.min_price = Some(20.0);
        config.max_price = Some(50.0);

        let cmd = SearchCommand::new(config);
        let result = cmd.execute_with_client(&client, "test").await;
        assert!(result.is_ok());

        let output = result.unwrap();
        // Should only contain the mid-priced product
        assert!(output.contains("B002"));
        assert!(!output.contains("B001")); // Too cheap
        assert!(!output.contains("B003")); // Too expensive
    }

    #[tokio::test]
    async fn test_search_command_max_results() {
        let html = make_search_html(&[
            ("B001", "Product 1", 10.0),
            ("B002", "Product 2", 20.0),
            ("B003", "Product 3", 30.0),
            ("B004", "Product 4", 40.0),
            ("B005", "Product 5", 50.0),
            ("B006", "Product 6", 60.0),
        ]);

        let client = MockAmazonClient::new(vec![html]);
        let mut config = make_test_config();
        config.max_results = 3;

        let cmd = SearchCommand::new(config);
        let result = cmd.execute_with_client(&client, "test").await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.contains("B001"));
        assert!(output.contains("B002"));
        assert!(output.contains("B003"));
        assert!(!output.contains("B004")); // Exceeds max_results
    }

    #[tokio::test]
    async fn test_search_command_json_format() {
        let html = make_search_html(&[("B001", "Test Product", 19.99)]);

        let client = MockAmazonClient::new(vec![html]);
        let mut config = make_test_config();
        config.format = OutputFormat::Json;

        let cmd = SearchCommand::new(config);
        let result = cmd.execute_with_client(&client, "test").await;
        assert!(result.is_ok());

        let output = result.unwrap();
        // JSON format should start with [
        assert!(output.starts_with('['));
        assert!(output.contains("B001"));
    }

    #[tokio::test]
    async fn test_search_command_pagination() {
        let page1 = make_search_html(&[("B001", "Product 1", 10.0), ("B002", "Product 2", 20.0)]);

        // Add next page indicator
        let page1_with_next =
            page1.replace("</body>", r#"<a class="s-pagination-next">Next</a></body>"#);

        let page2 = make_search_html(&[("B003", "Product 3", 30.0)]);

        let client = MockAmazonClient::new(vec![page1_with_next, page2]);
        let mut config = make_test_config();
        config.max_results = 10; // Allow pagination

        let cmd = SearchCommand::new(config);
        let result = cmd.execute_with_client(&client, "test").await;
        assert!(result.is_ok());

        // Should have fetched multiple pages
        assert!(client.call_count() >= 2);
    }

    #[tokio::test]
    async fn test_search_command_keyword_filter() {
        let html = make_search_html(&[
            ("B001", "Gaming Mouse RGB", 29.99),
            ("B002", "Office Mouse", 19.99),
            ("B003", "Gaming Keyboard", 49.99),
        ]);

        let client = MockAmazonClient::new(vec![html]);
        let mut config = make_test_config();
        config.keywords = vec!["Gaming".to_string(), "Mouse".to_string()];

        let cmd = SearchCommand::new(config);
        let result = cmd.execute_with_client(&client, "test").await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.contains("B001")); // Gaming Mouse RGB
        assert!(!output.contains("B002")); // Office Mouse - missing "Gaming"
        assert!(!output.contains("B003")); // Gaming Keyboard - missing "Mouse"
    }

    #[tokio::test]
    async fn test_search_command_exclude_keywords() {
        let html = make_search_html(&[
            ("B001", "New Mouse", 29.99),
            ("B002", "Refurbished Mouse", 19.99),
            ("B003", "Used Mouse", 9.99),
        ]);

        let client = MockAmazonClient::new(vec![html]);
        let mut config = make_test_config();
        config.exclude_keywords = vec!["Refurbished".to_string(), "Used".to_string()];

        let cmd = SearchCommand::new(config);
        let result = cmd.execute_with_client(&client, "test").await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.contains("B001")); // New Mouse
        assert!(!output.contains("B002")); // Refurbished
        assert!(!output.contains("B003")); // Used
    }
}
