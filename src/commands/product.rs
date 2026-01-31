//! Product lookup command implementation.

use crate::amazon::{AmazonClient, AmazonSearch, Parser, Product};
use crate::config::Config;
use crate::format::Formatter;
use anyhow::{Context, Result};
use tracing::info;

/// Executes a product lookup by ASIN.
pub struct ProductCommand {
    config: Config,
}

impl ProductCommand {
    /// Creates a new product command.
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// Fetches a product by ASIN and returns formatted output.
    pub async fn execute(&self, asin: &str) -> Result<String> {
        let client =
            AmazonClient::new(&self.config).await.context("Failed to create HTTP client")?;

        self.execute_with_client(&client, asin).await
    }

    /// Fetches a product with a provided client (for testing).
    pub async fn execute_with_client(
        &self,
        client: &impl AmazonSearch,
        asin: &str,
    ) -> Result<String> {
        // Validate ASIN format (10 alphanumeric characters)
        let asin = asin.trim().to_uppercase();
        if asin.len() != 10 || !asin.chars().all(|c| c.is_ascii_alphanumeric()) {
            anyhow::bail!(
                "Invalid ASIN format: '{}'. ASIN should be 10 alphanumeric characters.",
                asin
            );
        }

        info!("Looking up product: {}", asin);

        let parser = Parser::new(client.region());
        let html = client.product(&asin).await?;
        let product = parser.parse_product_page(&html, &asin)?;

        // Format output
        let formatter = Formatter::new(self.config.format);
        Ok(formatter.format_product(&product))
    }

    /// Fetches multiple products by ASIN.
    pub async fn execute_batch(&self, asins: &[String]) -> Result<String> {
        let client =
            AmazonClient::new(&self.config).await.context("Failed to create HTTP client")?;

        self.execute_batch_with_client(&client, asins).await
    }

    /// Fetches multiple products with a provided client (for testing).
    pub async fn execute_batch_with_client(
        &self,
        client: &impl AmazonSearch,
        asins: &[String],
    ) -> Result<String> {
        let parser = Parser::new(client.region());
        let mut products: Vec<Product> = Vec::new();

        for asin in asins {
            let asin = asin.trim().to_uppercase();
            if asin.len() != 10 || !asin.chars().all(|c| c.is_ascii_alphanumeric()) {
                eprintln!("Skipping invalid ASIN: {}", asin);
                continue;
            }

            info!("Looking up product: {}", asin);

            match client.product(&asin).await {
                Ok(html) => match parser.parse_product_page(&html, &asin) {
                    Ok(product) => products.push(product),
                    Err(e) => eprintln!("Failed to parse {}: {}", asin, e),
                },
                Err(e) => eprintln!("Failed to fetch {}: {}", asin, e),
            }
        }

        let formatter = Formatter::new(self.config.format);
        Ok(formatter.format_products(&products))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::amazon::Region;
    use crate::config::OutputFormat;
    use async_trait::async_trait;

    /// Mock Amazon client for testing.
    struct MockAmazonClient {
        product_html: String,
        should_fail: bool,
        region: Region,
    }

    impl MockAmazonClient {
        fn new(product_html: String) -> Self {
            Self { product_html, should_fail: false, region: Region::Us }
        }

        fn failing() -> Self {
            Self { product_html: String::new(), should_fail: true, region: Region::Us }
        }
    }

    #[async_trait]
    impl AmazonSearch for MockAmazonClient {
        async fn search(&self, _query: &str, _page: u32) -> Result<String> {
            Ok("<html></html>".to_string())
        }

        async fn product(&self, _asin: &str) -> Result<String> {
            if self.should_fail {
                anyhow::bail!("Simulated network error")
            } else {
                Ok(self.product_html.clone())
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
            max_results: 20,
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

    fn make_product_html(title: &str, price: f64) -> String {
        format!(
            r#"<html><body>
                <span id="productTitle">{}</span>
                <div id="corePrice_feature_div">
                    <span class="a-price"><span class="a-offscreen">${:.2}</span></span>
                </div>
                <div id="availability"><span>In Stock</span></div>
            </body></html>"#,
            title, price
        )
    }

    #[test]
    fn test_asin_validation() {
        // Valid ASINs
        assert!("B08N5WRWNW".chars().all(|c| c.is_ascii_alphanumeric()));
        assert_eq!("B08N5WRWNW".len(), 10);

        // Invalid: too short
        assert_ne!("B08N5".len(), 10);

        // Invalid: special characters
        assert!(!"B08N5-WRWNW".chars().all(|c| c.is_ascii_alphanumeric()));
    }

    #[tokio::test]
    async fn test_product_command_basic() {
        let html = make_product_html("Amazing Test Product", 29.99);
        let client = MockAmazonClient::new(html);
        let config = make_test_config();
        let cmd = ProductCommand::new(config);

        let result = cmd.execute_with_client(&client, "B08N5WRWNW").await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.contains("Amazing Test Product"));
        assert!(output.contains("B08N5WRWNW"));
    }

    #[tokio::test]
    async fn test_product_command_invalid_asin_short() {
        let client = MockAmazonClient::new(String::new());
        let config = make_test_config();
        let cmd = ProductCommand::new(config);

        let result = cmd.execute_with_client(&client, "SHORT").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid ASIN"));
    }

    #[tokio::test]
    async fn test_product_command_invalid_asin_long() {
        let client = MockAmazonClient::new(String::new());
        let config = make_test_config();
        let cmd = ProductCommand::new(config);

        let result = cmd.execute_with_client(&client, "TOOLONGASIN12345").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid ASIN"));
    }

    #[tokio::test]
    async fn test_product_command_invalid_asin_special_chars() {
        let client = MockAmazonClient::new(String::new());
        let config = make_test_config();
        let cmd = ProductCommand::new(config);

        let result = cmd.execute_with_client(&client, "B08N5!@#$%").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid ASIN"));
    }

    #[tokio::test]
    async fn test_product_command_asin_trimmed() {
        let html = make_product_html("Test Product", 19.99);
        let client = MockAmazonClient::new(html);
        let config = make_test_config();
        let cmd = ProductCommand::new(config);

        // ASIN with whitespace should be trimmed
        let result = cmd.execute_with_client(&client, "  B08N5WRWNW  ").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_product_command_asin_uppercase() {
        let html = make_product_html("Test Product", 19.99);
        let client = MockAmazonClient::new(html);
        let config = make_test_config();
        let cmd = ProductCommand::new(config);

        // lowercase ASIN should be uppercased
        let result = cmd.execute_with_client(&client, "b08n5wrwnw").await;
        assert!(result.is_ok());
        assert!(result.unwrap().contains("B08N5WRWNW"));
    }

    #[tokio::test]
    async fn test_product_command_json_format() {
        let html = make_product_html("Test Product", 19.99);
        let client = MockAmazonClient::new(html);
        let mut config = make_test_config();
        config.format = OutputFormat::Json;

        let cmd = ProductCommand::new(config);
        let result = cmd.execute_with_client(&client, "B08N5WRWNW").await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.starts_with('{'));
        assert!(output.contains("\"asin\""));
    }

    #[tokio::test]
    async fn test_product_command_markdown_format() {
        let html = make_product_html("Test Product", 19.99);
        let client = MockAmazonClient::new(html);
        let mut config = make_test_config();
        config.format = OutputFormat::Markdown;

        let cmd = ProductCommand::new(config);
        let result = cmd.execute_with_client(&client, "B08N5WRWNW").await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.contains("## Test Product"));
        assert!(output.contains("**ASIN:**"));
    }

    #[tokio::test]
    async fn test_product_command_batch_success() {
        let html = make_product_html("Test Product", 19.99);
        let client = MockAmazonClient::new(html);
        let config = make_test_config();
        let cmd = ProductCommand::new(config);

        let asins = vec!["B08N5WRWNW".to_string(), "B08N5WRWNX".to_string()];
        let result = cmd.execute_batch_with_client(&client, &asins).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_product_command_batch_skips_invalid() {
        let html = make_product_html("Test Product", 19.99);
        let client = MockAmazonClient::new(html);
        let config = make_test_config();
        let cmd = ProductCommand::new(config);

        let asins = vec![
            "B08N5WRWNW".to_string(),
            "SHORT".to_string(), // Invalid
            "B08N5WRWNX".to_string(),
        ];
        let result = cmd.execute_batch_with_client(&client, &asins).await;
        assert!(result.is_ok());
        // Invalid ASIN should be skipped, others processed
    }

    #[tokio::test]
    async fn test_product_command_network_error() {
        let client = MockAmazonClient::failing();
        let config = make_test_config();
        let cmd = ProductCommand::new(config);

        let result = cmd.execute_with_client(&client, "B08N5WRWNW").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("network error"));
    }
}
