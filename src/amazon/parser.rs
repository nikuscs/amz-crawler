//! HTML parser for Amazon search results and product pages.

use crate::amazon::models::{Price, PriceRange, Product, Rating, SearchResults};
use crate::amazon::regions::Region;
use crate::amazon::selectors::{errors, product, search};
use anyhow::{Context, Result};
use scraper::{ElementRef, Html};
use tracing::{debug, trace, warn};

/// Parser for Amazon HTML pages.
pub struct Parser {
    region: Region,
}

impl Parser {
    /// Creates a new parser for the given region.
    pub fn new(region: Region) -> Self {
        Self { region }
    }

    /// Parses search results HTML into structured data.
    pub fn parse_search(&self, html: &str, query: &str, page: u32) -> Result<SearchResults> {
        let document = Html::parse_document(html);

        // Check for error pages first
        self.check_for_errors(&document)?;

        let mut results = SearchResults::new(query, self.region.to_string());
        results.page = page;

        // Parse total results count
        results.total_results = self.parse_total_results(&document);

        // Parse each product card
        for element in document.select(&search::RESULT) {
            match self.parse_product_card(element) {
                Ok(Some(product)) => {
                    trace!("Parsed product: {} - {}", product.asin, product.title);
                    results.products.push(product);
                }
                Ok(None) => {
                    // Empty ASIN, skip (ad placeholder or similar)
                    trace!("Skipping empty result card");
                }
                Err(e) => {
                    warn!("Failed to parse product card: {}", e);
                    // Continue parsing other products
                }
            }
        }

        // Check for next page
        results.has_more = document.select(&search::NEXT_PAGE).next().is_some();

        debug!(
            "Parsed {} products from page {} (has_more: {})",
            results.products.len(),
            page,
            results.has_more
        );

        Ok(results)
    }

    /// Parses a single product page by ASIN.
    pub fn parse_product_page(&self, html: &str, asin: &str) -> Result<Product> {
        let document = Html::parse_document(html);

        // Check for error pages
        self.check_for_errors(&document)?;

        // Parse title
        let title = document
            .select(&product::TITLE)
            .next()
            .map(|e| e.text().collect::<String>().trim().to_string())
            .context("Could not find product title")?;

        // Parse price
        let price = self.parse_product_page_price(&document);

        // Parse rating
        let rating = self.parse_product_page_rating(&document);

        // Parse image
        let image_url = document.select(&product::IMAGE).next().and_then(|e| {
            e.value().attr("src").or_else(|| e.value().attr("data-old-hires")).map(String::from)
        });

        // Parse brand
        let brand = document.select(&product::BRAND).next().map(|e| {
            let text = e.text().collect::<String>();
            text.trim()
                .trim_start_matches("Brand:")
                .trim_start_matches("Visit the")
                .trim_end_matches("Store")
                .trim()
                .to_string()
        });

        // Check availability
        let in_stock = document.select(&product::AVAILABILITY).next().is_some_and(|e| {
            let text = e.text().collect::<String>().to_lowercase();
            text.contains("in stock") || text.contains("available")
        });

        // Check for Prime
        let is_prime = document.select(&product::PRIME).next().is_some();

        // Check for Amazon's Choice
        let is_amazon_choice = document.select(&product::AMAZON_CHOICE).next().is_some();

        Ok(Product {
            asin: asin.to_string(),
            title,
            url: format!("{}/dp/{}", self.region.base_url(), asin),
            image_url,
            price,
            rating,
            is_sponsored: false, // Product pages aren't sponsored
            is_prime,
            is_amazon_choice,
            in_stock,
            brand,
        })
    }

    /// Checks for CAPTCHA, error pages, or rate limiting.
    fn check_for_errors(&self, document: &Html) -> Result<()> {
        // Check for CAPTCHA
        if document.select(&errors::CAPTCHA).next().is_some() {
            anyhow::bail!(
                "CAPTCHA detected. Amazon is blocking requests. \
                Try using a proxy or waiting before retrying."
            );
        }

        // Check for dog page (503 error page)
        if document.select(&errors::DOG_PAGE).next().is_some() {
            anyhow::bail!(
                "Amazon error page detected (503). \
                The service may be temporarily unavailable."
            );
        }

        Ok(())
    }

    /// Parses a single product card from search results.
    fn parse_product_card(&self, element: ElementRef) -> Result<Option<Product>> {
        // Get ASIN
        let asin = match element.value().attr(search::ASIN_ATTR) {
            Some(asin) if !asin.is_empty() => asin.to_string(),
            _ => return Ok(None), // Skip cards without ASIN
        };

        // Parse title
        let title = element
            .select(&search::TITLE)
            .next()
            .map(|e| e.text().collect::<String>().trim().to_string())
            .unwrap_or_else(|| "Unknown".to_string());

        // Parse URL
        let url = element
            .select(&search::TITLE_LINK)
            .next()
            .and_then(|e| e.value().attr("href"))
            .map(|href| {
                if href.starts_with("http") {
                    href.to_string()
                } else {
                    format!("{}{}", self.region.base_url(), href)
                }
            })
            .unwrap_or_else(|| format!("{}/dp/{}", self.region.base_url(), asin));

        // Parse image
        let image_url = element
            .select(&search::IMAGE)
            .next()
            .and_then(|e| e.value().attr("src").map(String::from));

        // Parse price
        let price = self.parse_search_price(element);

        // Parse rating
        let rating = self.parse_search_rating(element);

        // Check for sponsored
        let is_sponsored = self.is_sponsored(element);

        // Check for Prime
        let is_prime = element.select(&search::PRIME_BADGE).next().is_some();

        // Check for Amazon's Choice
        let is_amazon_choice = self.is_amazon_choice(element);

        // Parse brand
        let brand = element
            .select(&search::BRAND)
            .next()
            .map(|e| e.text().collect::<String>().trim().trim_start_matches("by ").to_string());

        // Check stock (assume in stock if price is shown)
        let in_stock = price.is_some();

        Ok(Some(Product {
            asin,
            title,
            url,
            image_url,
            price,
            rating,
            is_sponsored,
            is_prime,
            is_amazon_choice,
            in_stock,
            brand,
        }))
    }

    /// Parses price from a search result card.
    fn parse_search_price(&self, element: ElementRef) -> Option<Price> {
        // Try to get the offscreen price text first (most reliable)
        let current_text =
            element.select(&search::PRICE_CURRENT).next().map(|e| e.text().collect::<String>())?;

        // Check for "See price in cart"
        if current_text.to_lowercase().contains("cart")
            || current_text.to_lowercase().contains("see price")
        {
            return Some(Price::hidden(self.region.currency()));
        }

        let current = self.parse_price_value(&current_text)?;

        // Check for original price
        let original = element
            .select(&search::PRICE_ORIGINAL)
            .next()
            .and_then(|e| self.parse_price_value(&e.text().collect::<String>()));

        // Check for price range
        let range = self.detect_price_range(element, current);

        Some(Price {
            current,
            original,
            currency: self.region.currency().to_string(),
            range,
            is_hidden: false,
        })
    }

    /// Parses price from a product detail page.
    fn parse_product_page_price(&self, document: &Html) -> Option<Price> {
        let current_text =
            document.select(&product::PRICE).next().map(|e| e.text().collect::<String>())?;

        let current = self.parse_price_value(&current_text)?;

        let original = document
            .select(&product::PRICE_ORIGINAL)
            .next()
            .and_then(|e| self.parse_price_value(&e.text().collect::<String>()));

        Some(Price {
            current,
            original,
            currency: self.region.currency().to_string(),
            range: None,
            is_hidden: false,
        })
    }

    /// Parses a price value from text, handling different regional formats.
    fn parse_price_value(&self, text: &str) -> Option<f64> {
        let cleaned: String = text
            .chars()
            .filter(|c| c.is_ascii_digit() || *c == '.' || *c == ',' || *c == '-')
            .collect();

        if cleaned.is_empty() {
            return None;
        }

        // Handle price ranges like "10-20" or "10 - 20"
        if cleaned.contains('-') {
            let parts: Vec<&str> = cleaned.split('-').collect();
            if let Some(first) = parts.first() {
                return self.parse_single_price(first);
            }
        }

        self.parse_single_price(&cleaned)
    }

    /// Parses a single price number.
    fn parse_single_price(&self, text: &str) -> Option<f64> {
        let cleaned = text.trim();
        if cleaned.is_empty() {
            return None;
        }

        // Determine decimal separator based on region
        let normalized = if self.region.uses_comma_decimal() {
            // EU format: 1.234,56 -> 1234.56
            cleaned.replace('.', "").replace(',', ".")
        } else {
            // US format: 1,234.56 -> 1234.56
            cleaned.replace(',', "")
        };

        normalized.parse().ok()
    }

    /// Detects if there's a price range.
    fn detect_price_range(&self, element: ElementRef, min: f64) -> Option<PriceRange> {
        // Check for explicit price range container
        if element.select(&search::PRICE_RANGE).next().is_some() {
            // Try to find max price from second price element
            let prices: Vec<_> = element.select(&search::PRICE_CURRENT).collect();
            if prices.len() >= 2 {
                if let Some(max) = self.parse_price_value(&prices[1].text().collect::<String>()) {
                    if max > min {
                        return Some(PriceRange { min, max: Some(max) });
                    }
                }
            }
        }
        None
    }

    /// Parses rating from a search result card.
    fn parse_search_rating(&self, element: ElementRef) -> Option<Rating> {
        // Parse star rating (e.g., "4.5 out of 5 stars")
        let stars_text =
            element.select(&search::RATING_STARS).next().map(|e| e.text().collect::<String>())?;

        let stars = self.parse_stars(&stars_text)?;

        // Parse review count
        let count_text = element
            .select(&search::RATING_COUNT)
            .next()
            .map(|e| e.text().collect::<String>())
            .unwrap_or_default();

        let review_count = self.parse_review_count(&count_text);

        Some(Rating::new(stars, review_count))
    }

    /// Parses rating from a product detail page.
    fn parse_product_page_rating(&self, document: &Html) -> Option<Rating> {
        let stars_text =
            document.select(&product::RATING).next().map(|e| e.text().collect::<String>())?;

        let stars = self.parse_stars(&stars_text)?;

        let count_text = document
            .select(&product::REVIEW_COUNT)
            .next()
            .map(|e| e.text().collect::<String>())
            .unwrap_or_default();

        let review_count = self.parse_review_count(&count_text);

        Some(Rating::new(stars, review_count))
    }

    /// Extracts star rating from text like "4.5 out of 5 stars".
    fn parse_stars(&self, text: &str) -> Option<f32> {
        // Look for a decimal number (e.g., "4.5")
        let re_pattern = text.split_whitespace().next()?.replace(',', "."); // Handle "4,5" format

        re_pattern.parse().ok()
    }

    /// Extracts review count from text like "1,234" or "1.234 ratings".
    fn parse_review_count(&self, text: &str) -> u32 {
        let cleaned: String = text.chars().filter(|c| c.is_ascii_digit()).collect();

        cleaned.parse().unwrap_or(0)
    }

    /// Checks if a product card is sponsored.
    fn is_sponsored(&self, element: ElementRef) -> bool {
        // Check for sponsored selector
        if element.select(&search::SPONSORED).next().is_some() {
            return true;
        }

        // Fallback: check for "Sponsored" text in the card
        let text = element.text().collect::<String>().to_lowercase();
        text.contains("sponsored")
    }

    /// Checks if a product has Amazon's Choice badge.
    fn is_amazon_choice(&self, element: ElementRef) -> bool {
        // Check for badge selector
        if element.select(&search::AMAZON_CHOICE).next().is_some() {
            return true;
        }

        // Fallback: check for "Amazon's Choice" text
        let text = element.text().collect::<String>();
        text.contains("Amazon's Choice") || text.contains("Amazon Choice")
    }

    /// Parses total results count from page.
    fn parse_total_results(&self, document: &Html) -> Option<u32> {
        let text =
            document.select(&search::TOTAL_RESULTS).next().map(|e| e.text().collect::<String>())?;

        // Extract number from text like "1-48 of over 10,000 results"
        let cleaned: String =
            text.split("of").nth(1)?.chars().filter(|c| c.is_ascii_digit()).collect();

        cleaned.parse().ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Price parsing tests

    #[test]
    fn test_parse_price_us() {
        let parser = Parser::new(Region::Us);
        assert_eq!(parser.parse_price_value("$29.99"), Some(29.99));
        assert_eq!(parser.parse_price_value("$1,234.56"), Some(1234.56));
        assert_eq!(parser.parse_price_value("29.99"), Some(29.99));
        assert_eq!(parser.parse_price_value("$0.99"), Some(0.99));
        assert_eq!(parser.parse_price_value("$10"), Some(10.0));
    }

    #[test]
    fn test_parse_price_eu() {
        let parser = Parser::new(Region::De);
        assert_eq!(parser.parse_price_value("29,99 €"), Some(29.99));
        assert_eq!(parser.parse_price_value("1.234,56 €"), Some(1234.56));
        assert_eq!(parser.parse_price_value("€29,99"), Some(29.99));
        assert_eq!(parser.parse_price_value("0,99€"), Some(0.99));
    }

    #[test]
    fn test_parse_price_other_regions() {
        // UK uses period decimal
        let parser = Parser::new(Region::Uk);
        assert_eq!(parser.parse_price_value("£29.99"), Some(29.99));

        // France uses comma decimal
        let parser = Parser::new(Region::Fr);
        assert_eq!(parser.parse_price_value("29,99 €"), Some(29.99));

        // Japan uses period decimal (no decimals typically)
        let parser = Parser::new(Region::Jp);
        assert_eq!(parser.parse_price_value("¥2,999"), Some(2999.0));
    }

    #[test]
    fn test_parse_price_with_range() {
        let parser = Parser::new(Region::Us);
        // Price ranges like "$10 - $20" should return the first price
        assert_eq!(parser.parse_price_value("$10 - $20"), Some(10.0));
        assert_eq!(parser.parse_price_value("10-20"), Some(10.0));
    }

    #[test]
    fn test_parse_price_empty() {
        let parser = Parser::new(Region::Us);
        assert_eq!(parser.parse_price_value(""), None);
        assert_eq!(parser.parse_price_value("   "), None);
        assert_eq!(parser.parse_price_value("N/A"), None);
    }

    #[test]
    fn test_parse_single_price_empty() {
        let parser = Parser::new(Region::Us);
        assert_eq!(parser.parse_single_price(""), None);
        assert_eq!(parser.parse_single_price("   "), None);
    }

    // Star rating parsing tests

    #[test]
    fn test_parse_stars() {
        let parser = Parser::new(Region::Us);
        assert_eq!(parser.parse_stars("4.5 out of 5 stars"), Some(4.5));
        assert_eq!(parser.parse_stars("4,5 von 5 Sternen"), Some(4.5));
        assert_eq!(parser.parse_stars("5.0 out of 5 stars"), Some(5.0));
        assert_eq!(parser.parse_stars("1 out of 5 stars"), Some(1.0));
    }

    #[test]
    fn test_parse_stars_edge_cases() {
        let parser = Parser::new(Region::Us);
        assert_eq!(parser.parse_stars(""), None);
    }

    // Review count parsing tests

    #[test]
    fn test_parse_review_count() {
        let parser = Parser::new(Region::Us);
        assert_eq!(parser.parse_review_count("1,234 ratings"), 1234);
        assert_eq!(parser.parse_review_count("1.234 Bewertungen"), 1234);
        assert_eq!(parser.parse_review_count("50 reviews"), 50);
        assert_eq!(parser.parse_review_count("1"), 1);
    }

    #[test]
    fn test_parse_review_count_edge_cases() {
        let parser = Parser::new(Region::Us);
        assert_eq!(parser.parse_review_count(""), 0);
        assert_eq!(parser.parse_review_count("no reviews"), 0);
    }

    // HTML parsing tests

    #[test]
    fn test_check_for_errors_clean_page() {
        let parser = Parser::new(Region::Us);
        let html = "<html><body><h1>Normal page</h1></body></html>";
        let document = Html::parse_document(html);
        assert!(parser.check_for_errors(&document).is_ok());
    }

    #[test]
    fn test_check_for_errors_captcha() {
        let parser = Parser::new(Region::Us);
        let html =
            r#"<html><body><form action="/errors/validateCaptcha">CAPTCHA</form></body></html>"#;
        let document = Html::parse_document(html);
        let result = parser.check_for_errors(&document);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("CAPTCHA"));
    }

    #[test]
    fn test_check_for_errors_dog_page() {
        let parser = Parser::new(Region::Us);
        // The selector uses [alt*='dog'] which is case-sensitive
        let html = r#"<html><body><img alt="Sorry, the dog ate this page"></body></html>"#;
        let document = Html::parse_document(html);
        let result = parser.check_for_errors(&document);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("503"));
    }

    #[test]
    fn test_parse_search_empty_results() {
        let parser = Parser::new(Region::Us);
        let html = r#"<html><body><div id="search"></div></body></html>"#;
        let results = parser.parse_search(html, "test query", 1).unwrap();
        assert_eq!(results.query, "test query");
        assert_eq!(results.region, "us");
        assert_eq!(results.page, 1);
        assert!(results.products.is_empty());
        assert!(!results.has_more);
    }

    #[test]
    fn test_parse_search_with_captcha() {
        let parser = Parser::new(Region::Us);
        let html = r#"<html><body><form action="/errors/validateCaptcha"></form></body></html>"#;
        let result = parser.parse_search(html, "test", 1);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_product_page_missing_title() {
        let parser = Parser::new(Region::Us);
        let html = r#"<html><body><div id="dp"></div></body></html>"#;
        let result = parser.parse_product_page(html, "B08N5WRWNW");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("title"));
    }

    #[test]
    fn test_parse_product_page_minimal() {
        let parser = Parser::new(Region::Us);
        let html = r#"
            <html><body>
                <span id="productTitle">Test Product</span>
            </body></html>
        "#;
        let product = parser.parse_product_page(html, "B08N5WRWNW").unwrap();
        assert_eq!(product.asin, "B08N5WRWNW");
        assert_eq!(product.title, "Test Product");
        assert!(product.price.is_none());
        assert!(product.rating.is_none());
        assert!(!product.in_stock);
    }

    #[test]
    fn test_parser_new() {
        let parser = Parser::new(Region::Uk);
        // Just verify it constructs without panicking
        assert_eq!(parser.region, Region::Uk);
    }

    // Test all region price parsing

    #[test]
    fn test_parse_price_all_eu_regions() {
        // All EU regions use comma as decimal separator
        for region in [Region::De, Region::Fr, Region::Es, Region::It, Region::Nl, Region::Pl] {
            let parser = Parser::new(region);
            assert_eq!(
                parser.parse_price_value("29,99"),
                Some(29.99),
                "Failed for region {:?}",
                region
            );
            assert_eq!(
                parser.parse_price_value("1.234,56"),
                Some(1234.56),
                "Failed for region {:?}",
                region
            );
        }
    }

    #[test]
    fn test_parse_price_all_period_regions() {
        // Regions that use period as decimal separator
        for region in [Region::Us, Region::Uk, Region::Ca, Region::Au, Region::Jp, Region::In] {
            let parser = Parser::new(region);
            assert_eq!(
                parser.parse_price_value("29.99"),
                Some(29.99),
                "Failed for region {:?}",
                region
            );
            assert_eq!(
                parser.parse_price_value("1,234.56"),
                Some(1234.56),
                "Failed for region {:?}",
                region
            );
        }
    }
}
