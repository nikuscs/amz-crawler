//! HTML parser for TropicalPrice pages.

use super::models::{CountryPrice, PriceComparison, TropicalProduct};
use anyhow::Result;
use scraper::{Html, Selector};
use std::sync::LazyLock;
use tracing::{debug, warn};

// Selectors for TropicalPrice HTML parsing
mod selectors {
    use super::*;

    pub static SEARCH_ITEM: LazyLock<Selector> = LazyLock::new(|| Selector::parse("li").unwrap());

    pub static PRODUCT_LINK: LazyLock<Selector> =
        LazyLock::new(|| Selector::parse("a[href*='/product/']").unwrap());

    pub static PRODUCT_TITLE: LazyLock<Selector> = LazyLock::new(|| Selector::parse("h2").unwrap());

    pub static PRICE_LINK: LazyLock<Selector> =
        LazyLock::new(|| Selector::parse("a.price").unwrap());

    pub static PRICE_TABLE: LazyLock<Selector> =
        LazyLock::new(|| Selector::parse("table.product-table").unwrap());

    pub static TABLE_ROW: LazyLock<Selector> = LazyLock::new(|| Selector::parse("tr").unwrap());

    pub static FLAG_CELL: LazyLock<Selector> =
        LazyLock::new(|| Selector::parse("td.product-table-flag").unwrap());

    pub static FLAG_IMG: LazyLock<Selector> = LazyLock::new(|| Selector::parse("img").unwrap());

    pub static PRICE_CELL: LazyLock<Selector> =
        LazyLock::new(|| Selector::parse("td.product-table-price").unwrap());

    pub static PRICE_AMOUNT: LazyLock<Selector> =
        LazyLock::new(|| Selector::parse("span.product-table-price-amount").unwrap());
}

/// Parses TropicalPrice search results HTML.
pub fn parse_search_results(html: &str, max_results: usize) -> Result<Vec<TropicalProduct>> {
    let document = Html::parse_document(html);
    let mut products = Vec::new();

    for item in document.select(&selectors::SEARCH_ITEM).take(max_results) {
        // Find product link
        let Some(link) = item.select(&selectors::PRODUCT_LINK).next() else {
            continue;
        };

        let Some(href) = link.value().attr("href") else {
            continue;
        };

        // Extract ASIN from URL
        let Some(asin) = extract_asin(href) else {
            continue;
        };

        // Extract title
        let title = item
            .select(&selectors::PRODUCT_TITLE)
            .next()
            .map(|e| e.text().collect::<String>().trim().to_string())
            .unwrap_or_else(|| "Unknown".to_string());

        // Extract price
        let price = item.select(&selectors::PRICE_LINK).next().and_then(|e| {
            let text = e.text().collect::<String>();
            parse_eur_price(&text)
        });

        products.push(TropicalProduct {
            asin,
            title,
            price,
            currency: "EUR".to_string(),
            url: format!("https://tropicalprice.com{}", href),
        });
    }

    debug!("Parsed {} products from TropicalPrice search", products.len());
    Ok(products)
}

/// Parses TropicalPrice product comparison page HTML.
pub fn parse_price_comparison(html: &str, asin: &str) -> Result<Option<PriceComparison>> {
    let document = Html::parse_document(html);

    // Extract title
    let title = document
        .select(&selectors::PRODUCT_TITLE)
        .next()
        .map(|e| e.text().collect::<String>().trim().to_string())
        .unwrap_or_else(|| "Unknown Product".to_string());

    // Find price table
    let Some(table) = document.select(&selectors::PRICE_TABLE).next() else {
        warn!("No price table found for ASIN {}", asin);
        return Ok(None);
    };

    let mut prices = Vec::new();

    for row in table.select(&selectors::TABLE_ROW) {
        // Extract country from flag image
        let Some(flag_cell) = row.select(&selectors::FLAG_CELL).next() else {
            continue;
        };

        let Some(img) = flag_cell.select(&selectors::FLAG_IMG).next() else {
            continue;
        };

        let Some(country) = img.value().attr("alt") else {
            continue;
        };

        let country = country.to_uppercase();

        // Extract price
        let Some(price_cell) = row.select(&selectors::PRICE_CELL).next() else {
            continue;
        };

        let Some(price_span) = price_cell.select(&selectors::PRICE_AMOUNT).next() else {
            continue;
        };

        let price_text = price_span.text().collect::<String>();
        let is_marketplace = price_text.contains("**");

        let Some(price) = parse_eur_price(&price_text) else {
            continue;
        };

        // Determine Amazon domain
        let amazon_domain = match country.as_str() {
            "UK" | "CO.UK" => "co.uk",
            _ => &country.to_lowercase(),
        };

        prices.push(CountryPrice {
            country: country.clone(),
            price,
            currency: "EUR".to_string(),
            is_marketplace,
            amazon_url: format!("https://www.amazon.{}/dp/{}", amazon_domain, asin),
        });
    }

    if prices.is_empty() {
        return Ok(None);
    }

    // Sort by price (cheapest first)
    prices.sort_by(|a, b| a.price.partial_cmp(&b.price).unwrap_or(std::cmp::Ordering::Equal));

    let total_stores = prices.len();

    Ok(Some(PriceComparison { asin: asin.to_string(), title, prices, total_stores }))
}

/// Extracts ASIN from TropicalPrice URL.
fn extract_asin(url: &str) -> Option<String> {
    // Look for /product/ASIN pattern
    let re = regex_lite::Regex::new(r"/product/([A-Z0-9]{10})").ok()?;
    re.captures(url).map(|c| c[1].to_string())
}

/// Parses EUR price from text like "€99.99" or "99,99 €".
fn parse_eur_price(text: &str) -> Option<f64> {
    // Remove currency symbols and whitespace
    let cleaned: String =
        text.chars().filter(|c| c.is_ascii_digit() || *c == '.' || *c == ',').collect();

    if cleaned.is_empty() {
        return None;
    }

    // Detect format based on separator positions
    let last_comma = cleaned.rfind(',');
    let last_period = cleaned.rfind('.');

    let normalized = match (last_comma, last_period) {
        // Only comma -> EU decimal (99,99 -> 99.99)
        (Some(_), None) => cleaned.replace(',', "."),
        // Only period -> US decimal (99.99 -> 99.99)
        (None, Some(_)) => cleaned,
        // Both: check which comes last (decimal separator is always last)
        (Some(c), Some(p)) => {
            if c > p {
                // EU format: 1.234,56 -> 1234.56
                cleaned.replace('.', "").replace(',', ".")
            } else {
                // US format: 1,234.56 -> 1234.56
                cleaned.replace(',', "")
            }
        }
        // Neither -> just digits
        (None, None) => cleaned,
    };

    normalized.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_asin() {
        assert_eq!(extract_asin("/product/B08N5WRWNW"), Some("B08N5WRWNW".to_string()));
        assert_eq!(
            extract_asin("https://tropicalprice.com/product/B08N5WRWNW?ref=test"),
            Some("B08N5WRWNW".to_string())
        );
        assert_eq!(extract_asin("/other/path"), None);
        assert_eq!(extract_asin(""), None);
        assert_eq!(extract_asin("/product/"), None);
        assert_eq!(extract_asin("/product/SHORT"), None); // Too short for ASIN
    }

    #[test]
    fn test_parse_eur_price() {
        assert_eq!(parse_eur_price("€99.99"), Some(99.99));
        assert_eq!(parse_eur_price("99,99 €"), Some(99.99));
        assert_eq!(parse_eur_price("1.234,56 €"), Some(1234.56));
        assert_eq!(parse_eur_price("€1,234.56"), Some(1234.56));
        assert_eq!(parse_eur_price("invalid"), None);
    }

    #[test]
    fn test_parse_eur_price_edge_cases() {
        assert_eq!(parse_eur_price(""), None);
        assert_eq!(parse_eur_price("   "), None);
        assert_eq!(parse_eur_price("€"), None);
        assert_eq!(parse_eur_price("N/A"), None);
    }

    #[test]
    fn test_parse_eur_price_integers() {
        assert_eq!(parse_eur_price("€100"), Some(100.0));
        assert_eq!(parse_eur_price("50€"), Some(50.0));
        assert_eq!(parse_eur_price("1000"), Some(1000.0));
    }

    #[test]
    fn test_parse_eur_price_with_spaces() {
        assert_eq!(parse_eur_price("€ 99.99"), Some(99.99));
        assert_eq!(parse_eur_price("99.99 €"), Some(99.99));
        assert_eq!(parse_eur_price("  €50  "), Some(50.0));
    }

    #[test]
    fn test_parse_eur_price_marketplace_indicator() {
        // Prices with ** for marketplace listings
        assert_eq!(parse_eur_price("€99.99**"), Some(99.99));
        assert_eq!(parse_eur_price("**€50.00"), Some(50.0));
    }

    #[test]
    fn test_parse_search_results_empty() {
        let html = "<html><body><ul></ul></body></html>";
        let results = parse_search_results(html, 10).unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_parse_search_results_no_products() {
        let html = r#"<html><body>
            <ul>
                <li><span>Not a product</span></li>
            </ul>
        </body></html>"#;
        let results = parse_search_results(html, 10).unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_parse_search_results_with_product() {
        let html = r#"<html><body>
            <ul>
                <li>
                    <a href="/product/B08N5WRWNW">Product Link</a>
                    <h2>Test Product Title</h2>
                    <a class="price">€49.99</a>
                </li>
            </ul>
        </body></html>"#;
        let results = parse_search_results(html, 10).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].asin, "B08N5WRWNW");
        assert_eq!(results[0].title, "Test Product Title");
        assert_eq!(results[0].price, Some(49.99));
        assert_eq!(results[0].currency, "EUR");
        assert!(results[0].url.contains("tropicalprice.com"));
    }

    #[test]
    fn test_parse_search_results_max_limit() {
        let html = r#"<html><body>
            <ul>
                <li><a href="/product/B0000000A1">Link</a><h2>Product 1</h2></li>
                <li><a href="/product/B0000000A2">Link</a><h2>Product 2</h2></li>
                <li><a href="/product/B0000000A3">Link</a><h2>Product 3</h2></li>
            </ul>
        </body></html>"#;
        let results = parse_search_results(html, 2).unwrap();
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_parse_price_comparison_no_table() {
        let html = r#"<html><body>
            <h2>Product Title</h2>
        </body></html>"#;
        let result = parse_price_comparison(html, "B08N5WRWNW").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_price_comparison_empty_table() {
        let html = r#"<html><body>
            <h2>Product Title</h2>
            <table class="product-table"></table>
        </body></html>"#;
        let result = parse_price_comparison(html, "B08N5WRWNW").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_price_comparison_with_prices() {
        let html = r#"<html><body>
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
        </body></html>"#;
        let result = parse_price_comparison(html, "B08N5WRWNW").unwrap();
        assert!(result.is_some());
        let comparison = result.unwrap();
        assert_eq!(comparison.asin, "B08N5WRWNW");
        assert_eq!(comparison.title, "Test Product");
        assert_eq!(comparison.total_stores, 2);
        // Prices should be sorted (cheapest first)
        assert_eq!(comparison.prices[0].price, 49.99);
        assert_eq!(comparison.prices[0].country, "DE");
        assert_eq!(comparison.prices[1].price, 54.99);
        assert_eq!(comparison.prices[1].country, "FR");
    }

    #[test]
    fn test_parse_price_comparison_uk_domain() {
        let html = r#"<html><body>
            <h2>Test Product</h2>
            <table class="product-table">
                <tr>
                    <td class="product-table-flag"><img alt="UK"></td>
                    <td class="product-table-price"><span class="product-table-price-amount">£39.99</span></td>
                </tr>
            </table>
        </body></html>"#;
        let result = parse_price_comparison(html, "B08N5WRWNW").unwrap();
        assert!(result.is_some());
        let comparison = result.unwrap();
        assert!(comparison.prices[0].amazon_url.contains("amazon.co.uk"));
    }

    #[test]
    fn test_parse_price_comparison_marketplace() {
        let html = r#"<html><body>
            <h2>Test Product</h2>
            <table class="product-table">
                <tr>
                    <td class="product-table-flag"><img alt="DE"></td>
                    <td class="product-table-price"><span class="product-table-price-amount">€49.99**</span></td>
                </tr>
            </table>
        </body></html>"#;
        let result = parse_price_comparison(html, "B08N5WRWNW").unwrap();
        assert!(result.is_some());
        let comparison = result.unwrap();
        assert!(comparison.prices[0].is_marketplace);
    }
}
