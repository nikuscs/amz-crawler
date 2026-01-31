//! Keyword-based title filtering.

use super::Filter;
use crate::amazon::Product;

/// Filters products by keywords in the title.
pub struct KeywordFilter {
    /// Keywords that must appear in the title.
    required: Vec<String>,
    /// Keywords that must NOT appear in the title.
    excluded: Vec<String>,
}

impl KeywordFilter {
    /// Creates a new keyword filter.
    pub fn new(required: Vec<String>, excluded: Vec<String>) -> Self {
        Self {
            required: required.into_iter().map(|k| k.to_lowercase()).collect(),
            excluded: excluded.into_iter().map(|k| k.to_lowercase()).collect(),
        }
    }

    /// Creates a filter with only required keywords.
    pub fn required(keywords: Vec<String>) -> Self {
        Self::new(keywords, Vec::new())
    }

    /// Creates a filter with only excluded keywords.
    pub fn excluded(keywords: Vec<String>) -> Self {
        Self::new(Vec::new(), keywords)
    }
}

impl Filter for KeywordFilter {
    fn matches(&self, product: &Product) -> bool {
        let title = product.title.to_lowercase();

        // Check required keywords (all must be present)
        for keyword in &self.required {
            if !title.contains(keyword) {
                return false;
            }
        }

        // Check excluded keywords (none must be present)
        for keyword in &self.excluded {
            if title.contains(keyword) {
                return false;
            }
        }

        true
    }

    fn description(&self) -> String {
        let mut parts = Vec::new();

        if !self.required.is_empty() {
            parts.push(format!("Must contain: {}", self.required.join(", ")));
        }

        if !self.excluded.is_empty() {
            parts.push(format!("Must not contain: {}", self.excluded.join(", ")));
        }

        if parts.is_empty() {
            "Keywords: any".to_string()
        } else {
            parts.join("; ")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_product(title: &str) -> Product {
        Product {
            asin: "TEST".to_string(),
            title: title.to_string(),
            url: "https://amazon.com/dp/TEST".to_string(),
            image_url: None,
            price: None,
            rating: None,
            is_sponsored: false,
            is_prime: false,
            is_amazon_choice: false,
            in_stock: true,
            brand: None,
        }
    }

    #[test]
    fn test_required_keywords() {
        let filter = KeywordFilter::required(vec!["wireless".to_string(), "mouse".to_string()]);

        assert!(filter.matches(&make_product("Wireless Gaming Mouse")));
        assert!(filter.matches(&make_product("WIRELESS MOUSE pad"))); // Case insensitive
        assert!(!filter.matches(&make_product("Wireless Keyboard")));
        assert!(!filter.matches(&make_product("Gaming Mouse"))); // Missing "wireless"
    }

    #[test]
    fn test_excluded_keywords() {
        let filter = KeywordFilter::excluded(vec!["refurbished".to_string(), "used".to_string()]);

        assert!(filter.matches(&make_product("New Wireless Mouse")));
        assert!(!filter.matches(&make_product("Refurbished Laptop")));
        assert!(!filter.matches(&make_product("Used Gaming Chair")));
    }

    #[test]
    fn test_both_required_and_excluded() {
        let filter =
            KeywordFilter::new(vec!["laptop".to_string()], vec!["refurbished".to_string()]);

        assert!(filter.matches(&make_product("Gaming Laptop 15 inch")));
        assert!(!filter.matches(&make_product("Desktop Computer")));
        assert!(!filter.matches(&make_product("Refurbished Laptop")));
    }

    #[test]
    fn test_empty_keywords() {
        let filter = KeywordFilter::new(Vec::new(), Vec::new());
        assert!(filter.matches(&make_product("Anything at all")));
    }

    #[test]
    fn test_case_insensitivity() {
        let filter = KeywordFilter::required(vec!["GAMING".to_string()]);
        assert!(filter.matches(&make_product("gaming mouse")));
        assert!(filter.matches(&make_product("Gaming Mouse")));
        assert!(filter.matches(&make_product("GAMING MOUSE")));
    }

    #[test]
    fn test_partial_match() {
        let filter = KeywordFilter::required(vec!["wire".to_string()]);
        assert!(filter.matches(&make_product("Wireless Mouse"))); // "wire" is in "wireless"
        assert!(filter.matches(&make_product("Wired Keyboard")));
    }

    #[test]
    fn test_description_required_only() {
        let filter = KeywordFilter::required(vec!["gaming".to_string(), "mouse".to_string()]);
        let desc = filter.description();
        assert!(desc.contains("Must contain:"));
        assert!(desc.contains("gaming"));
        assert!(desc.contains("mouse"));
    }

    #[test]
    fn test_description_excluded_only() {
        let filter = KeywordFilter::excluded(vec!["refurbished".to_string()]);
        let desc = filter.description();
        assert!(desc.contains("Must not contain:"));
        assert!(desc.contains("refurbished"));
    }

    #[test]
    fn test_description_both() {
        let filter =
            KeywordFilter::new(vec!["laptop".to_string()], vec!["refurbished".to_string()]);
        let desc = filter.description();
        assert!(desc.contains("Must contain:"));
        assert!(desc.contains("laptop"));
        assert!(desc.contains("Must not contain:"));
        assert!(desc.contains("refurbished"));
    }

    #[test]
    fn test_description_empty() {
        let filter = KeywordFilter::new(Vec::new(), Vec::new());
        assert_eq!(filter.description(), "Keywords: any");
    }

    #[test]
    fn test_keywords_stored_lowercase() {
        let filter =
            KeywordFilter::new(vec!["UPPERCASE".to_string()], vec!["EXCLUDED".to_string()]);
        // The keywords should match case-insensitively
        assert!(filter.matches(&make_product("uppercase product")));
        assert!(!filter.matches(&make_product("excluded product")));
    }
}
