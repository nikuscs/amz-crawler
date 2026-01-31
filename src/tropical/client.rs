//! HTTP client for TropicalPrice requests.

use super::models::{PriceComparison, TropicalProduct};
use super::parser;
use anyhow::{Context, Result};
use async_trait::async_trait;
use std::time::Duration;
use tracing::{debug, info};
use wreq::Client;

const TROPICALPRICE_BASE: &str = "https://tropicalprice.com";

/// Trait for TropicalPrice operations - enables mocking for tests.
#[async_trait]
pub trait TropicalSearch: Send + Sync {
    /// Searches TropicalPrice for products.
    async fn search(&self, query: &str, max_results: usize) -> Result<Vec<TropicalProduct>>;

    /// Gets price comparison for a specific ASIN across EU stores.
    async fn compare(&self, asin: &str) -> Result<Option<PriceComparison>>;
}

/// TropicalPrice HTTP client.
pub struct TropicalClient {
    client: Client,
    base_url: String,
}

impl TropicalClient {
    /// Creates a new TropicalPrice client.
    pub fn new() -> Result<Self> {
        Self::with_base_url(TROPICALPRICE_BASE.to_string())
    }

    /// Creates a new TropicalPrice client with a custom base URL (for testing).
    pub fn with_base_url(base_url: String) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(15))
            .connect_timeout(Duration::from_secs(10))
            .build()?;

        Ok(Self { client, base_url })
    }

    /// Internal method to fetch HTML from a URL.
    async fn fetch(&self, url: &str) -> Result<String> {
        debug!("GET {}", url);

        let response = self
            .client
            .get(url)
            .header("Accept", "*/*")
            .header("Accept-Language", "en-US,en;q=0.9")
            .header("X-Requested-With", "XMLHttpRequest")
            .send()
            .await
            .context("Failed to send request")?;

        if !response.status().is_success() {
            anyhow::bail!("TropicalPrice returned status: {}", response.status());
        }

        response.text().await.context("Failed to read response body")
    }
}

#[async_trait]
impl TropicalSearch for TropicalClient {
    async fn search(&self, query: &str, max_results: usize) -> Result<Vec<TropicalProduct>> {
        let url = format!("{}/search/es?q={}&p=1", self.base_url, urlencoding::encode(query));

        info!("Searching TropicalPrice: {}", query);
        let html = self.fetch(&url).await?;
        parser::parse_search_results(&html, max_results)
    }

    async fn compare(&self, asin: &str) -> Result<Option<PriceComparison>> {
        // Validate ASIN
        let asin = asin.trim().to_uppercase();
        if asin.len() != 10 || !asin.chars().all(|c| c.is_ascii_alphanumeric()) {
            anyhow::bail!("Invalid ASIN format: {}", asin);
        }

        let url = format!("{}/product/{}", self.base_url, asin);

        info!("Comparing prices for ASIN: {}", asin);
        let html = self.fetch(&url).await?;
        parser::parse_price_comparison(&html, &asin)
    }
}

impl Default for TropicalClient {
    fn default() -> Self {
        Self::new().expect("Failed to create TropicalClient")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_search_success() {
        let mock_server = MockServer::start().await;

        let html = r#"
            <html><body>
                <ul>
                    <li>
                        <a href="/product/B08N5WRWNW">Product Link</a>
                        <h2>Test Product Title</h2>
                        <a class="price">€49.99</a>
                    </li>
                </ul>
            </body></html>
        "#;

        Mock::given(method("GET"))
            .and(path("/search/es"))
            .and(query_param("q", "test"))
            .and(query_param("p", "1"))
            .respond_with(ResponseTemplate::new(200).set_body_string(html))
            .mount(&mock_server)
            .await;

        let client = TropicalClient::with_base_url(mock_server.uri()).unwrap();
        let results = client.search("test", 10).await.unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].asin, "B08N5WRWNW");
        assert_eq!(results[0].title, "Test Product Title");
        assert_eq!(results[0].price, Some(49.99));
    }

    #[tokio::test]
    async fn test_search_empty_results() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/search/es"))
            .respond_with(ResponseTemplate::new(200).set_body_string("<html><ul></ul></html>"))
            .mount(&mock_server)
            .await;

        let client = TropicalClient::with_base_url(mock_server.uri()).unwrap();
        let results = client.search("nonexistent", 10).await.unwrap();

        assert!(results.is_empty());
    }

    #[tokio::test]
    async fn test_search_error_500() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/search/es"))
            .respond_with(ResponseTemplate::new(500))
            .mount(&mock_server)
            .await;

        let client = TropicalClient::with_base_url(mock_server.uri()).unwrap();
        let result = client.search("test", 10).await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("500"));
    }

    #[tokio::test]
    async fn test_compare_success() {
        let mock_server = MockServer::start().await;

        let html = r#"
            <html><body>
                <h2>Test Product</h2>
                <table class="product-table">
                    <tr>
                        <td class="product-table-flag"><img alt="DE"></td>
                        <td class="product-table-price"><span class="product-table-price-amount">€49.99</span></td>
                    </tr>
                    <tr>
                        <td class="product-table-flag"><img alt="FR"></td>
                        <td class="product-table-price"><span class="product-table-price-amount">€54.99</span></td>
                    </tr>
                </table>
            </body></html>
        "#;

        Mock::given(method("GET"))
            .and(path("/product/B08N5WRWNW"))
            .respond_with(ResponseTemplate::new(200).set_body_string(html))
            .mount(&mock_server)
            .await;

        let client = TropicalClient::with_base_url(mock_server.uri()).unwrap();
        let comparison = client.compare("B08N5WRWNW").await.unwrap();

        assert!(comparison.is_some());
        let comp = comparison.unwrap();
        assert_eq!(comp.asin, "B08N5WRWNW");
        assert_eq!(comp.title, "Test Product");
        assert_eq!(comp.prices.len(), 2);
        assert_eq!(comp.prices[0].country, "DE");
        assert_eq!(comp.prices[0].price, 49.99);
        assert_eq!(comp.prices[1].country, "FR");
        assert_eq!(comp.prices[1].price, 54.99);
    }

    #[tokio::test]
    async fn test_compare_no_prices() {
        let mock_server = MockServer::start().await;

        let html = r#"
            <html><body>
                <h2>Test Product</h2>
            </body></html>
        "#;

        Mock::given(method("GET"))
            .and(path("/product/B08N5WRWNW"))
            .respond_with(ResponseTemplate::new(200).set_body_string(html))
            .mount(&mock_server)
            .await;

        let client = TropicalClient::with_base_url(mock_server.uri()).unwrap();
        let comparison = client.compare("B08N5WRWNW").await.unwrap();

        assert!(comparison.is_none());
    }

    #[tokio::test]
    async fn test_compare_invalid_asin_too_short() {
        let client = TropicalClient::with_base_url("http://localhost".to_string()).unwrap();
        let result = client.compare("SHORT").await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid ASIN"));
    }

    #[tokio::test]
    async fn test_compare_invalid_asin_too_long() {
        let client = TropicalClient::with_base_url("http://localhost".to_string()).unwrap();
        let result = client.compare("TOOLONGASIN123").await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid ASIN"));
    }

    #[tokio::test]
    async fn test_compare_invalid_asin_special_chars() {
        let client = TropicalClient::with_base_url("http://localhost".to_string()).unwrap();
        let result = client.compare("B08N5!@#$%").await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid ASIN"));
    }

    #[tokio::test]
    async fn test_compare_asin_trimmed_and_uppercased() {
        let mock_server = MockServer::start().await;

        let html = r#"
            <html><body>
                <h2>Test Product</h2>
                <table class="product-table">
                    <tr>
                        <td class="product-table-flag"><img alt="DE"></td>
                        <td class="product-table-price"><span class="product-table-price-amount">€49.99</span></td>
                    </tr>
                </table>
            </body></html>
        "#;

        // The path should be uppercase even if lowercase is provided
        Mock::given(method("GET"))
            .and(path("/product/B08N5WRWNW"))
            .respond_with(ResponseTemplate::new(200).set_body_string(html))
            .mount(&mock_server)
            .await;

        let client = TropicalClient::with_base_url(mock_server.uri()).unwrap();
        // Provide lowercase with spaces
        let comparison = client.compare("  b08n5wrwnw  ").await.unwrap();

        assert!(comparison.is_some());
    }

    #[tokio::test]
    async fn test_compare_error_404() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/product/B08N5WRWNW"))
            .respond_with(ResponseTemplate::new(404))
            .mount(&mock_server)
            .await;

        let client = TropicalClient::with_base_url(mock_server.uri()).unwrap();
        let result = client.compare("B08N5WRWNW").await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("404"));
    }

    #[tokio::test]
    async fn test_compare_marketplace_indicator() {
        let mock_server = MockServer::start().await;

        let html = r#"
            <html><body>
                <h2>Test Product</h2>
                <table class="product-table">
                    <tr>
                        <td class="product-table-flag"><img alt="DE"></td>
                        <td class="product-table-price"><span class="product-table-price-amount">€49.99**</span></td>
                    </tr>
                </table>
            </body></html>
        "#;

        Mock::given(method("GET"))
            .and(path("/product/B08N5WRWNW"))
            .respond_with(ResponseTemplate::new(200).set_body_string(html))
            .mount(&mock_server)
            .await;

        let client = TropicalClient::with_base_url(mock_server.uri()).unwrap();
        let comparison = client.compare("B08N5WRWNW").await.unwrap().unwrap();

        assert!(comparison.prices[0].is_marketplace);
    }

    #[tokio::test]
    async fn test_search_with_special_characters() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/search/es"))
            .respond_with(ResponseTemplate::new(200).set_body_string("<html><ul></ul></html>"))
            .mount(&mock_server)
            .await;

        let client = TropicalClient::with_base_url(mock_server.uri()).unwrap();
        let results = client.search("iphone 15 pro", 10).await.unwrap();

        assert!(results.is_empty());
    }

    #[tokio::test]
    async fn test_search_max_results_limit() {
        let mock_server = MockServer::start().await;

        let html = r#"
            <html><body>
                <ul>
                    <li><a href="/product/B0000000A1">Link</a><h2>Product 1</h2></li>
                    <li><a href="/product/B0000000A2">Link</a><h2>Product 2</h2></li>
                    <li><a href="/product/B0000000A3">Link</a><h2>Product 3</h2></li>
                    <li><a href="/product/B0000000A4">Link</a><h2>Product 4</h2></li>
                    <li><a href="/product/B0000000A5">Link</a><h2>Product 5</h2></li>
                </ul>
            </body></html>
        "#;

        Mock::given(method("GET"))
            .and(path("/search/es"))
            .respond_with(ResponseTemplate::new(200).set_body_string(html))
            .mount(&mock_server)
            .await;

        let client = TropicalClient::with_base_url(mock_server.uri()).unwrap();
        let results = client.search("test", 3).await.unwrap();

        assert_eq!(results.len(), 3);
    }

    #[tokio::test]
    async fn test_new_client() {
        let client = TropicalClient::new();
        assert!(client.is_ok());
    }

    #[tokio::test]
    async fn test_default_client() {
        let client: TropicalClient = Default::default();
        assert_eq!(client.base_url, TROPICALPRICE_BASE);
    }

    #[tokio::test]
    async fn test_compare_uk_domain() {
        let mock_server = MockServer::start().await;

        let html = r#"
            <html><body>
                <h2>Test Product</h2>
                <table class="product-table">
                    <tr>
                        <td class="product-table-flag"><img alt="UK"></td>
                        <td class="product-table-price"><span class="product-table-price-amount">£39.99</span></td>
                    </tr>
                </table>
            </body></html>
        "#;

        Mock::given(method("GET"))
            .and(path("/product/B08N5WRWNW"))
            .respond_with(ResponseTemplate::new(200).set_body_string(html))
            .mount(&mock_server)
            .await;

        let client = TropicalClient::with_base_url(mock_server.uri()).unwrap();
        let comparison = client.compare("B08N5WRWNW").await.unwrap().unwrap();

        assert!(comparison.prices[0].amazon_url.contains("amazon.co.uk"));
    }
}
