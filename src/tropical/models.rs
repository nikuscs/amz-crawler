//! Data models for TropicalPrice responses.

use serde::{Deserialize, Serialize};

/// Country code to flag emoji mapping.
pub fn country_flag(code: &str) -> &'static str {
    match code.to_uppercase().as_str() {
        "DE" => "🇩🇪",
        "ES" => "🇪🇸",
        "FR" => "🇫🇷",
        "IT" => "🇮🇹",
        "NL" => "🇳🇱",
        "BE" => "🇧🇪",
        "AT" => "🇦🇹",
        "PL" => "🇵🇱",
        "SE" => "🇸🇪",
        "CO.UK" | "UK" => "🇬🇧",
        _ => "🏳️",
    }
}

/// A product from TropicalPrice search results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TropicalProduct {
    /// Amazon ASIN
    pub asin: String,
    /// Product title
    pub title: String,
    /// Lowest price in EUR
    pub price: Option<f64>,
    /// Currency (always EUR for TropicalPrice)
    pub currency: String,
    /// TropicalPrice URL
    pub url: String,
}

/// Price for a specific country/Amazon store.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CountryPrice {
    /// Country code (DE, ES, FR, IT, etc.)
    pub country: String,
    /// Price in EUR
    pub price: f64,
    /// Currency (EUR)
    pub currency: String,
    /// Whether this is a marketplace seller (not Amazon directly)
    pub is_marketplace: bool,
    /// Direct Amazon URL for this country
    pub amazon_url: String,
}

impl CountryPrice {
    /// Returns the country flag emoji.
    pub fn flag(&self) -> &'static str {
        country_flag(&self.country)
    }
}

/// Price comparison across EU Amazon stores for a single ASIN.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceComparison {
    /// Amazon ASIN
    pub asin: String,
    /// Product title
    pub title: String,
    /// Prices sorted by cheapest first
    pub prices: Vec<CountryPrice>,
    /// Total number of stores with prices
    pub total_stores: usize,
}

impl PriceComparison {
    /// Returns the cheapest price option.
    pub fn cheapest(&self) -> Option<&CountryPrice> {
        self.prices.first()
    }

    /// Returns the most expensive price option.
    pub fn most_expensive(&self) -> Option<&CountryPrice> {
        self.prices.last()
    }

    /// Calculates savings compared to the most expensive store.
    pub fn max_savings(&self) -> Option<f64> {
        match (self.cheapest(), self.most_expensive()) {
            (Some(cheap), Some(expensive)) => Some(expensive.price - cheap.price),
            _ => None,
        }
    }

    /// Calculates savings percentage compared to most expensive.
    pub fn max_savings_percent(&self) -> Option<f64> {
        match (self.cheapest(), self.most_expensive()) {
            (Some(cheap), Some(expensive)) if expensive.price > 0.0 => {
                Some((expensive.price - cheap.price) / expensive.price * 100.0)
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_country_price(country: &str, price: f64) -> CountryPrice {
        CountryPrice {
            country: country.to_string(),
            price,
            currency: "EUR".to_string(),
            is_marketplace: false,
            amazon_url: format!("https://www.amazon.{}/dp/TEST", country.to_lowercase()),
        }
    }

    // Country flag tests

    #[test]
    fn test_country_flags_all() {
        assert_eq!(country_flag("DE"), "🇩🇪");
        assert_eq!(country_flag("ES"), "🇪🇸");
        assert_eq!(country_flag("FR"), "🇫🇷");
        assert_eq!(country_flag("IT"), "🇮🇹");
        assert_eq!(country_flag("NL"), "🇳🇱");
        assert_eq!(country_flag("BE"), "🇧🇪");
        assert_eq!(country_flag("AT"), "🇦🇹");
        assert_eq!(country_flag("PL"), "🇵🇱");
        assert_eq!(country_flag("SE"), "🇸🇪");
        assert_eq!(country_flag("UK"), "🇬🇧");
        assert_eq!(country_flag("CO.UK"), "🇬🇧");
    }

    #[test]
    fn test_country_flags_case_insensitive() {
        assert_eq!(country_flag("de"), "🇩🇪");
        assert_eq!(country_flag("De"), "🇩🇪");
        assert_eq!(country_flag("DE"), "🇩🇪");
        assert_eq!(country_flag("uk"), "🇬🇧");
        assert_eq!(country_flag("co.uk"), "🇬🇧");
    }

    #[test]
    fn test_country_flags_unknown() {
        assert_eq!(country_flag("XX"), "🏳️");
        assert_eq!(country_flag(""), "🏳️");
        assert_eq!(country_flag("US"), "🏳️");
    }

    // CountryPrice tests

    #[test]
    fn test_country_price_flag() {
        let price = make_country_price("DE", 49.99);
        assert_eq!(price.flag(), "🇩🇪");

        let price = make_country_price("UK", 39.99);
        assert_eq!(price.flag(), "🇬🇧");
    }

    #[test]
    fn test_country_price_serde() {
        let price = CountryPrice {
            country: "DE".to_string(),
            price: 49.99,
            currency: "EUR".to_string(),
            is_marketplace: true,
            amazon_url: "https://www.amazon.de/dp/TEST".to_string(),
        };

        let json = serde_json::to_string(&price).unwrap();
        assert!(json.contains("DE"));
        assert!(json.contains("49.99"));
        assert!(json.contains("true"));

        let parsed: CountryPrice = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.country, "DE");
        assert_eq!(parsed.price, 49.99);
        assert!(parsed.is_marketplace);
    }

    // PriceComparison tests

    #[test]
    fn test_price_comparison() {
        let comparison = PriceComparison {
            asin: "B08N5WRWNW".to_string(),
            title: "Test Product".to_string(),
            prices: vec![make_country_price("DE", 89.99), make_country_price("FR", 99.99)],
            total_stores: 2,
        };

        assert_eq!(comparison.cheapest().unwrap().country, "DE");
        assert_eq!(comparison.most_expensive().unwrap().country, "FR");
        assert_eq!(comparison.max_savings(), Some(10.0));
    }

    #[test]
    fn test_price_comparison_empty() {
        let comparison = PriceComparison {
            asin: "TEST".to_string(),
            title: "Test".to_string(),
            prices: Vec::new(),
            total_stores: 0,
        };

        assert!(comparison.cheapest().is_none());
        assert!(comparison.most_expensive().is_none());
        assert!(comparison.max_savings().is_none());
        assert!(comparison.max_savings_percent().is_none());
    }

    #[test]
    fn test_price_comparison_single_price() {
        let comparison = PriceComparison {
            asin: "TEST".to_string(),
            title: "Test".to_string(),
            prices: vec![make_country_price("DE", 50.0)],
            total_stores: 1,
        };

        assert_eq!(comparison.cheapest().unwrap().country, "DE");
        assert_eq!(comparison.most_expensive().unwrap().country, "DE");
        assert_eq!(comparison.max_savings(), Some(0.0));
        assert_eq!(comparison.max_savings_percent(), Some(0.0));
    }

    #[test]
    fn test_max_savings_percent() {
        let comparison = PriceComparison {
            asin: "TEST".to_string(),
            title: "Test".to_string(),
            prices: vec![make_country_price("DE", 80.0), make_country_price("FR", 100.0)],
            total_stores: 2,
        };

        // (100 - 80) / 100 * 100 = 20%
        assert_eq!(comparison.max_savings_percent(), Some(20.0));
    }

    #[test]
    fn test_max_savings_percent_zero_expensive() {
        let comparison = PriceComparison {
            asin: "TEST".to_string(),
            title: "Test".to_string(),
            prices: vec![make_country_price("DE", 0.0), make_country_price("FR", 0.0)],
            total_stores: 2,
        };

        // Avoid division by zero
        assert!(comparison.max_savings_percent().is_none());
    }

    // TropicalProduct tests

    #[test]
    fn test_tropical_product_serde() {
        let product = TropicalProduct {
            asin: "B08N5WRWNW".to_string(),
            title: "Test Product".to_string(),
            price: Some(49.99),
            currency: "EUR".to_string(),
            url: "https://tropicalprice.com/product/B08N5WRWNW".to_string(),
        };

        let json = serde_json::to_string(&product).unwrap();
        assert!(json.contains("B08N5WRWNW"));
        assert!(json.contains("Test Product"));
        assert!(json.contains("49.99"));

        let parsed: TropicalProduct = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.asin, "B08N5WRWNW");
        assert_eq!(parsed.price, Some(49.99));
    }

    #[test]
    fn test_tropical_product_no_price() {
        let product = TropicalProduct {
            asin: "TEST".to_string(),
            title: "Test".to_string(),
            price: None,
            currency: "EUR".to_string(),
            url: "https://tropicalprice.com/product/TEST".to_string(),
        };

        let json = serde_json::to_string(&product).unwrap();
        let parsed: TropicalProduct = serde_json::from_str(&json).unwrap();
        assert!(parsed.price.is_none());
    }

    // PriceComparison serde test

    #[test]
    fn test_price_comparison_serde() {
        let comparison = PriceComparison {
            asin: "TEST".to_string(),
            title: "Test Product".to_string(),
            prices: vec![make_country_price("DE", 50.0), make_country_price("FR", 60.0)],
            total_stores: 2,
        };

        let json = serde_json::to_string(&comparison).unwrap();
        let parsed: PriceComparison = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.asin, "TEST");
        assert_eq!(parsed.prices.len(), 2);
        assert_eq!(parsed.total_stores, 2);
    }
}
