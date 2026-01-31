//! HTTP client for Amazon requests using wreq for TLS fingerprint emulation.

use crate::amazon::regions::Region;
use crate::config::Config;
use anyhow::{Context, Result};
use async_trait::async_trait;
use rand::Rng;
use std::time::Duration;
use tracing::{debug, info, warn};
use wreq::Client;
use wreq_util::Emulation;

/// Trait for Amazon search/product fetching - enables mocking for tests.
#[async_trait]
pub trait AmazonSearch: Send + Sync {
    /// Performs a search and returns the HTML response.
    async fn search(&self, query: &str, page: u32) -> Result<String>;

    /// Fetches a product page by ASIN.
    async fn product(&self, asin: &str) -> Result<String>;

    /// Returns the configured region.
    fn region(&self) -> Region;
}

/// Amazon HTTP client with browser impersonation and anti-bot measures.
pub struct AmazonClient {
    client: Client,
    region: Region,
    delay_ms: u64,
    delay_jitter_ms: u64,
    base_url: Option<String>,
}

impl AmazonClient {
    /// Creates a new Amazon client with the given configuration.
    pub async fn new(config: &Config) -> Result<Self> {
        Self::with_base_url(config, None).await
    }

    /// Creates a new Amazon client with an optional custom base URL (for testing).
    pub async fn with_base_url(config: &Config, base_url: Option<String>) -> Result<Self> {
        let mut builder = Client::builder()
            .cookie_store(true)
            .gzip(true)
            .brotli(true)
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(10));

        // Configure proxy if specified
        if let Some(proxy_url) = &config.proxy {
            debug!("Configuring proxy: {}", proxy_url);
            let proxy = wreq::Proxy::all(proxy_url).context("Failed to configure proxy")?;
            builder = builder.proxy(proxy);
        }

        let client = builder.build()?;

        Ok(Self {
            client,
            region: config.region,
            delay_ms: config.delay_ms,
            delay_jitter_ms: config.delay_jitter_ms,
            base_url,
        })
    }

    /// Returns the base URL (custom for testing, or region-based for production).
    fn base_url(&self) -> String {
        self.base_url.clone().unwrap_or_else(|| self.region.base_url())
    }

    /// Performs a GET request with all anti-bot measures.
    async fn get(&self, url: &str) -> Result<String> {
        // Add human-like delay with jitter
        self.delay().await;

        debug!("GET {}", url);

        let response = self
            .client
            .get(url)
            .emulation(Emulation::Chrome131)
            .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8")
            .header("Accept-Language", self.region.accept_language())
            .header("Accept-Encoding", "gzip, deflate, br")
            .header("Cache-Control", "no-cache")
            .header("Pragma", "no-cache")
            .header("Sec-Ch-Ua", "\"Chromium\";v=\"131\", \"Not_A Brand\";v=\"24\"")
            .header("Sec-Ch-Ua-Mobile", "?0")
            .header("Sec-Ch-Ua-Platform", "\"macOS\"")
            .header("Sec-Fetch-Dest", "document")
            .header("Sec-Fetch-Mode", "navigate")
            .header("Sec-Fetch-Site", "none")
            .header("Sec-Fetch-User", "?1")
            .header("Upgrade-Insecure-Requests", "1")
            .send()
            .await
            .context("Failed to send request")?;

        let status = response.status();
        debug!("Response status: {}", status);

        if status == 503 {
            warn!("Rate limited (503). Consider using a proxy or increasing delay.");
            anyhow::bail!("Rate limited by Amazon. Try increasing --delay or using a proxy.");
        }

        if !status.is_success() {
            anyhow::bail!("Request failed with status: {}", status);
        }

        // Check for redirect to different region
        let final_url = response.uri().to_string();
        if !final_url.contains(self.region.domain()) && self.base_url.is_none() {
            warn!(
                "Redirected to different domain: {}. Your IP may be associated with a different region.",
                final_url
            );
        }

        response.text().await.context("Failed to read response body")
    }

    /// Adds a random delay to mimic human behavior.
    async fn delay(&self) {
        if self.delay_ms == 0 {
            return;
        }

        let jitter = if self.delay_jitter_ms > 0 {
            rand::rng().random_range(0..=self.delay_jitter_ms)
        } else {
            0
        };

        let total_delay = self.delay_ms + jitter;
        debug!("Delaying {}ms", total_delay);
        tokio::time::sleep(Duration::from_millis(total_delay)).await;
    }

    /// Updates the delay settings.
    pub fn set_delay(&mut self, delay_ms: u64, jitter_ms: u64) {
        self.delay_ms = delay_ms;
        self.delay_jitter_ms = jitter_ms;
    }
}

#[async_trait]
impl AmazonSearch for AmazonClient {
    async fn search(&self, query: &str, page: u32) -> Result<String> {
        let url = format!("{}/s?k={}&page={}", self.base_url(), urlencoding::encode(query), page);

        info!("Searching: {} (page {})", query, page);
        self.get(&url).await
    }

    async fn product(&self, asin: &str) -> Result<String> {
        let url = format!("{}/dp/{}", self.base_url(), asin);

        info!("Fetching product: {}", asin);
        self.get(&url).await
    }

    fn region(&self) -> Region {
        self.region
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn make_test_config() -> Config {
        Config {
            region: Region::Us,
            proxy: None,
            delay_ms: 0,        // No delay for tests
            delay_jitter_ms: 0, // No jitter for tests
            max_results: 20,
            format: crate::config::OutputFormat::Table,
            min_price: None,
            max_price: None,
            min_rating: None,
            prime_only: false,
            no_sponsored: false,
            keywords: Vec::new(),
            exclude_keywords: Vec::new(),
        }
    }

    #[test]
    fn test_url_encoding() {
        let query = "rust programming book";
        let encoded = urlencoding::encode(query);
        assert_eq!(encoded, "rust%20programming%20book");
    }

    #[tokio::test]
    async fn test_search_success() {
        let mock_server = MockServer::start().await;

        let html = r#"
            <html><body>
                <div data-component-type="s-search-result" data-asin="B08N5WRWNW">
                    <h2><a href="/dp/B08N5WRWNW"><span>Test Product</span></a></h2>
                </div>
            </body></html>
        "#;

        Mock::given(method("GET"))
            .and(path("/s"))
            .respond_with(ResponseTemplate::new(200).set_body_string(html))
            .mount(&mock_server)
            .await;

        let config = make_test_config();
        let client = AmazonClient::with_base_url(&config, Some(mock_server.uri())).await.unwrap();

        let result = client.search("test query", 1).await;
        assert!(result.is_ok());
        let body = result.unwrap();
        assert!(body.contains("Test Product"));
        assert!(body.contains("B08N5WRWNW"));
    }

    #[tokio::test]
    async fn test_product_success() {
        let mock_server = MockServer::start().await;

        let html = r#"
            <html><body>
                <span id="productTitle">Amazing Product Title</span>
                <span class="a-price"><span class="a-offscreen">$29.99</span></span>
            </body></html>
        "#;

        Mock::given(method("GET"))
            .and(path("/dp/B08N5WRWNW"))
            .respond_with(ResponseTemplate::new(200).set_body_string(html))
            .mount(&mock_server)
            .await;

        let config = make_test_config();
        let client = AmazonClient::with_base_url(&config, Some(mock_server.uri())).await.unwrap();

        let result = client.product("B08N5WRWNW").await;
        assert!(result.is_ok());
        let body = result.unwrap();
        assert!(body.contains("Amazing Product Title"));
        assert!(body.contains("$29.99"));
    }

    #[tokio::test]
    async fn test_rate_limited_503() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/s"))
            .respond_with(ResponseTemplate::new(503))
            .mount(&mock_server)
            .await;

        let config = make_test_config();
        let client = AmazonClient::with_base_url(&config, Some(mock_server.uri())).await.unwrap();

        let result = client.search("test", 1).await;
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Rate limited"));
    }

    #[tokio::test]
    async fn test_http_error_404() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/dp/INVALIDASIN"))
            .respond_with(ResponseTemplate::new(404))
            .mount(&mock_server)
            .await;

        let config = make_test_config();
        let client = AmazonClient::with_base_url(&config, Some(mock_server.uri())).await.unwrap();

        let result = client.product("INVALIDASIN").await;
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("404"));
    }

    #[tokio::test]
    async fn test_http_error_500() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/s"))
            .respond_with(ResponseTemplate::new(500))
            .mount(&mock_server)
            .await;

        let config = make_test_config();
        let client = AmazonClient::with_base_url(&config, Some(mock_server.uri())).await.unwrap();

        let result = client.search("test", 1).await;
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("500"));
    }

    #[tokio::test]
    async fn test_empty_response() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/s"))
            .respond_with(ResponseTemplate::new(200).set_body_string(""))
            .mount(&mock_server)
            .await;

        let config = make_test_config();
        let client = AmazonClient::with_base_url(&config, Some(mock_server.uri())).await.unwrap();

        let result = client.search("test", 1).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_region_returned() {
        let config = make_test_config();
        let client = AmazonClient::with_base_url(&config, Some("http://localhost".to_string()))
            .await
            .unwrap();

        assert_eq!(client.region(), Region::Us);
    }

    #[tokio::test]
    async fn test_set_delay() {
        let config = make_test_config();
        let mut client = AmazonClient::with_base_url(&config, Some("http://localhost".to_string()))
            .await
            .unwrap();

        client.set_delay(1000, 500);
        assert_eq!(client.delay_ms, 1000);
        assert_eq!(client.delay_jitter_ms, 500);
    }

    #[tokio::test]
    async fn test_base_url_default() {
        let config = make_test_config();
        let client = AmazonClient::new(&config).await.unwrap();

        assert_eq!(client.base_url(), "https://www.amazon.com");
    }

    #[tokio::test]
    async fn test_base_url_custom() {
        let config = make_test_config();
        let client = AmazonClient::with_base_url(&config, Some("http://custom.url".to_string()))
            .await
            .unwrap();

        assert_eq!(client.base_url(), "http://custom.url");
    }

    #[tokio::test]
    async fn test_search_with_special_characters() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/s"))
            .respond_with(ResponseTemplate::new(200).set_body_string("<html></html>"))
            .mount(&mock_server)
            .await;

        let config = make_test_config();
        let client = AmazonClient::with_base_url(&config, Some(mock_server.uri())).await.unwrap();

        let result = client.search("rust & c++", 1).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_search_pagination() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/s"))
            .and(query_param("page", "5"))
            .respond_with(ResponseTemplate::new(200).set_body_string("<html>page 5</html>"))
            .mount(&mock_server)
            .await;

        let config = make_test_config();
        let client = AmazonClient::with_base_url(&config, Some(mock_server.uri())).await.unwrap();

        let result = client.search("test", 5).await;
        assert!(result.is_ok());
        assert!(result.unwrap().contains("page 5"));
    }

    #[tokio::test]
    async fn test_different_regions() {
        let mut config = make_test_config();
        config.region = Region::Uk;

        let client = AmazonClient::new(&config).await.unwrap();
        assert_eq!(client.region(), Region::Uk);
        assert_eq!(client.base_url(), "https://www.amazon.co.uk");
    }
}
