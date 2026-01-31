//! TropicalPrice comparison command implementation.

use crate::config::OutputFormat;
use crate::tropical::{PriceComparison, TropicalClient, TropicalProduct, TropicalSearch};
use anyhow::Result;
use tracing::info;

/// Executes a TropicalPrice search.
pub async fn search_tropical(
    query: &str,
    max_results: usize,
    format: OutputFormat,
) -> Result<String> {
    let client = TropicalClient::new()?;
    search_tropical_with_client(&client, query, max_results, format).await
}

/// Executes a TropicalPrice search with a provided client (for testing).
pub async fn search_tropical_with_client(
    client: &impl TropicalSearch,
    query: &str,
    max_results: usize,
    format: OutputFormat,
) -> Result<String> {
    let products = client.search(query, max_results).await?;

    info!("Found {} products on TropicalPrice", products.len());

    Ok(match format {
        OutputFormat::Json => serde_json::to_string_pretty(&products)?,
        _ => format_search_results(&products),
    })
}

/// Executes a price comparison for an ASIN.
pub async fn compare_prices(asin: &str, format: OutputFormat) -> Result<String> {
    let client = TropicalClient::new()?;
    compare_prices_with_client(&client, asin, format).await
}

/// Executes a price comparison with a provided client (for testing).
pub async fn compare_prices_with_client(
    client: &impl TropicalSearch,
    asin: &str,
    format: OutputFormat,
) -> Result<String> {
    match client.compare(asin).await? {
        Some(comparison) => {
            info!("Found prices from {} stores for {}", comparison.total_stores, asin);

            Ok(match format {
                OutputFormat::Json => serde_json::to_string_pretty(&comparison)?,
                _ => format_comparison(&comparison),
            })
        }
        None => {
            anyhow::bail!("No price data found for ASIN {} on TropicalPrice", asin);
        }
    }
}

/// Formats search results as a table.
fn format_search_results(products: &[TropicalProduct]) -> String {
    if products.is_empty() {
        return "No products found on TropicalPrice.".to_string();
    }

    let mut lines = Vec::new();

    lines.push("TropicalPrice Search Results:".to_string());
    lines.push("=".repeat(80));
    lines.push(format!("{:<3} {:<12} {:<10} {:<55}", "#", "ASIN", "Price", "Title"));
    lines.push("-".repeat(80));

    for (i, p) in products.iter().enumerate() {
        let price_str =
            p.price.map(|pr| format!("€{:.2}", pr)).unwrap_or_else(|| "N/A".to_string());

        let title =
            if p.title.len() > 52 { format!("{}...", &p.title[..52]) } else { p.title.clone() };

        lines.push(format!("{:<3} {:<12} {:<10} {:<55}", i + 1, p.asin, price_str, title));
    }

    lines.push(String::new());
    lines.push("💡 To compare EU prices: amz-crawler compare <ASIN>".to_string());
    lines.push("💡 TropicalPrice URL: https://tropicalprice.com/product/<ASIN>".to_string());

    lines.join("\n")
}

/// Formats price comparison as a readable output.
fn format_comparison(data: &PriceComparison) -> String {
    let mut lines = Vec::new();

    // Product title
    lines.push(format!("📦 {}", data.title));
    lines.push(String::new());

    // Best price summary
    if let Some(cheapest) = data.cheapest() {
        let marketplace = if cheapest.is_marketplace { " ⚠️" } else { "" };
        lines.push(format!(
            "Best at {} {}: €{:.2}{}",
            cheapest.flag(),
            cheapest.country,
            cheapest.price,
            marketplace
        ));
        lines.push(format!("🛒 {}", cheapest.amazon_url));
        lines.push(String::new());
    }

    // Price list with savings
    let cheapest_price = data.cheapest().map(|c| c.price).unwrap_or(0.0);

    for p in &data.prices {
        let savings_eur = p.price - cheapest_price;
        let savings_pct = if cheapest_price > 0.0 {
            (p.price - cheapest_price) / cheapest_price * 100.0
        } else {
            0.0
        };

        let marker = if savings_eur == 0.0 { "🏆" } else { "  " };
        let marketplace = if p.is_marketplace { " ⚠️" } else { "" };

        if savings_eur == 0.0 {
            lines.push(format!(
                "{}{} {}: €{:.2}{}",
                marker,
                p.flag(),
                p.country,
                p.price,
                marketplace
            ));
        } else {
            lines.push(format!(
                "{}{} {}: €{:.2} (+€{:.0}, +{:.0}%){}",
                marker,
                p.flag(),
                p.country,
                p.price,
                savings_eur,
                savings_pct,
                marketplace
            ));
        }
    }

    // Max savings summary
    if let (Some(savings), Some(pct)) = (data.max_savings(), data.max_savings_percent()) {
        if savings > 0.0 {
            lines.push(String::new());
            lines.push(format!("💰 Max savings: €{:.2} ({:.0}%)", savings, pct));
        }
    }

    // All store links at the end
    lines.push(String::new());
    lines.push("🔗 Links:".to_string());
    for p in &data.prices {
        lines.push(format!("   {} {}: {}", p.flag(), p.country, p.amazon_url));
    }

    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tropical::CountryPrice;
    use async_trait::async_trait;

    /// Mock TropicalPrice client for testing.
    struct MockTropicalClient {
        search_results: Vec<TropicalProduct>,
        comparison_result: Option<PriceComparison>,
        should_fail: bool,
    }

    impl MockTropicalClient {
        fn with_search_results(products: Vec<TropicalProduct>) -> Self {
            Self { search_results: products, comparison_result: None, should_fail: false }
        }

        fn with_comparison(comparison: PriceComparison) -> Self {
            Self {
                search_results: Vec::new(),
                comparison_result: Some(comparison),
                should_fail: false,
            }
        }

        fn empty() -> Self {
            Self { search_results: Vec::new(), comparison_result: None, should_fail: false }
        }

        fn failing() -> Self {
            Self { search_results: Vec::new(), comparison_result: None, should_fail: true }
        }
    }

    #[async_trait]
    impl TropicalSearch for MockTropicalClient {
        async fn search(&self, _query: &str, _max_results: usize) -> Result<Vec<TropicalProduct>> {
            if self.should_fail {
                anyhow::bail!("Simulated network error")
            } else {
                Ok(self.search_results.clone())
            }
        }

        async fn compare(&self, _asin: &str) -> Result<Option<PriceComparison>> {
            if self.should_fail {
                anyhow::bail!("Simulated network error")
            } else {
                Ok(self.comparison_result.clone())
            }
        }
    }

    fn make_test_product(asin: &str, title: &str, price: Option<f64>) -> TropicalProduct {
        TropicalProduct {
            asin: asin.to_string(),
            title: title.to_string(),
            price,
            currency: "EUR".to_string(),
            url: format!("https://tropicalprice.com/product/{}", asin),
        }
    }

    fn make_country_price(country: &str, price: f64, is_marketplace: bool) -> CountryPrice {
        CountryPrice {
            country: country.to_string(),
            price,
            currency: "EUR".to_string(),
            is_marketplace,
            amazon_url: format!("https://www.amazon.{}/dp/TEST", country.to_lowercase()),
        }
    }

    fn make_test_comparison() -> PriceComparison {
        PriceComparison {
            asin: "B08N5WRWNW".to_string(),
            title: "Test Product".to_string(),
            prices: vec![
                make_country_price("DE", 49.99, false),
                make_country_price("FR", 54.99, false),
                make_country_price("IT", 59.99, true),
            ],
            total_stores: 3,
        }
    }

    // Search tests

    #[tokio::test]
    async fn test_search_tropical_success() {
        let products = vec![
            make_test_product("B001", "Product One", Some(29.99)),
            make_test_product("B002", "Product Two", Some(39.99)),
        ];
        let client = MockTropicalClient::with_search_results(products);

        let result = search_tropical_with_client(&client, "test", 10, OutputFormat::Table).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.contains("TropicalPrice Search Results"));
        assert!(output.contains("B001"));
        assert!(output.contains("B002"));
        assert!(output.contains("€29.99"));
    }

    #[tokio::test]
    async fn test_search_tropical_empty() {
        let client = MockTropicalClient::empty();

        let result =
            search_tropical_with_client(&client, "nonexistent", 10, OutputFormat::Table).await;
        assert!(result.is_ok());
        assert!(result.unwrap().contains("No products found"));
    }

    #[tokio::test]
    async fn test_search_tropical_json_format() {
        let products = vec![make_test_product("B001", "Test Product", Some(29.99))];
        let client = MockTropicalClient::with_search_results(products);

        let result = search_tropical_with_client(&client, "test", 10, OutputFormat::Json).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.starts_with('['));
        assert!(output.contains("\"asin\""));
        assert!(output.contains("B001"));
    }

    #[tokio::test]
    async fn test_search_tropical_no_price() {
        let products = vec![make_test_product("B001", "No Price Product", None)];
        let client = MockTropicalClient::with_search_results(products);

        let result = search_tropical_with_client(&client, "test", 10, OutputFormat::Table).await;
        assert!(result.is_ok());
        assert!(result.unwrap().contains("N/A"));
    }

    #[tokio::test]
    async fn test_search_tropical_long_title() {
        let long_title = "This is a very long product title that should be truncated in the output because it exceeds the maximum length";
        let products = vec![make_test_product("B001", long_title, Some(29.99))];
        let client = MockTropicalClient::with_search_results(products);

        let result = search_tropical_with_client(&client, "test", 10, OutputFormat::Table).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.contains("..."));
    }

    #[tokio::test]
    async fn test_search_tropical_network_error() {
        let client = MockTropicalClient::failing();

        let result = search_tropical_with_client(&client, "test", 10, OutputFormat::Table).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("network error"));
    }

    // Compare tests

    #[tokio::test]
    async fn test_compare_prices_success() {
        let comparison = make_test_comparison();
        let client = MockTropicalClient::with_comparison(comparison);

        let result = compare_prices_with_client(&client, "B08N5WRWNW", OutputFormat::Table).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.contains("DE"));
        assert!(output.contains("FR"));
        assert!(output.contains("€49.99"));
        assert!(output.contains("🏆")); // Winner marker
    }

    #[tokio::test]
    async fn test_compare_prices_json_format() {
        let comparison = make_test_comparison();
        let client = MockTropicalClient::with_comparison(comparison);

        let result = compare_prices_with_client(&client, "B08N5WRWNW", OutputFormat::Json).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.starts_with('{'));
        assert!(output.contains("\"asin\""));
        assert!(output.contains("\"prices\""));
    }

    #[tokio::test]
    async fn test_compare_prices_not_found() {
        let client = MockTropicalClient::empty();

        let result = compare_prices_with_client(&client, "B08N5WRWNW", OutputFormat::Table).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("No price data"));
    }

    #[tokio::test]
    async fn test_compare_prices_marketplace_indicator() {
        let comparison = make_test_comparison();
        let client = MockTropicalClient::with_comparison(comparison);

        let result = compare_prices_with_client(&client, "B08N5WRWNW", OutputFormat::Table).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.contains("⚠️")); // Marketplace warning
    }

    #[tokio::test]
    async fn test_compare_prices_savings() {
        let comparison = make_test_comparison();
        let client = MockTropicalClient::with_comparison(comparison);

        let result = compare_prices_with_client(&client, "B08N5WRWNW", OutputFormat::Table).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.contains("Max savings"));
        assert!(output.contains("💰"));
    }

    #[tokio::test]
    async fn test_compare_prices_network_error() {
        let client = MockTropicalClient::failing();

        let result = compare_prices_with_client(&client, "B08N5WRWNW", OutputFormat::Table).await;
        assert!(result.is_err());
    }

    // Format function tests

    #[test]
    fn test_format_search_results_empty() {
        let output = format_search_results(&[]);
        assert_eq!(output, "No products found on TropicalPrice.");
    }

    #[test]
    fn test_format_search_results_with_products() {
        let products = vec![make_test_product("B001", "Test Product", Some(29.99))];
        let output = format_search_results(&products);

        assert!(output.contains("TropicalPrice Search Results"));
        assert!(output.contains("B001"));
        assert!(output.contains("€29.99"));
        assert!(output.contains("amz-crawler compare"));
    }

    #[test]
    fn test_format_comparison_single_store() {
        let comparison = PriceComparison {
            asin: "TEST".to_string(),
            title: "Test".to_string(),
            prices: vec![make_country_price("DE", 50.0, false)],
            total_stores: 1,
        };

        let output = format_comparison(&comparison);
        assert!(output.contains("DE"));
        assert!(output.contains("€50.00"));
        assert!(output.contains("🏆")); // Should be winner
    }

    #[test]
    fn test_format_comparison_with_savings() {
        let comparison = make_test_comparison();
        let output = format_comparison(&comparison);

        assert!(output.contains("DE")); // Cheapest
        assert!(output.contains("FR")); // More expensive
        assert!(output.contains("+")); // Savings indicator
    }
}
