//! Output formatting for products (table, JSON, markdown, CSV).

use crate::amazon::Product;
use crate::config::OutputFormat;

/// Formats products for output.
pub struct Formatter {
    format: OutputFormat,
}

impl Formatter {
    /// Creates a new formatter.
    pub fn new(format: OutputFormat) -> Self {
        Self { format }
    }

    /// Formats a single product.
    pub fn format_product(&self, product: &Product) -> String {
        match self.format {
            OutputFormat::Json => self.json_single(product),
            OutputFormat::Table => self.table_single(product),
            OutputFormat::Markdown => self.markdown_single(product),
            OutputFormat::Csv => self.csv_products(std::slice::from_ref(product)),
        }
    }

    /// Formats multiple products.
    pub fn format_products(&self, products: &[Product]) -> String {
        if products.is_empty() {
            return match self.format {
                OutputFormat::Json => "[]".to_string(),
                OutputFormat::Csv => self.csv_header(),
                _ => "No products found.".to_string(),
            };
        }

        match self.format {
            OutputFormat::Json => self.json_products(products),
            OutputFormat::Table => self.table_products(products),
            OutputFormat::Markdown => self.markdown_products(products),
            OutputFormat::Csv => self.csv_products(products),
        }
    }

    // JSON formatting

    fn json_single(&self, product: &Product) -> String {
        serde_json::to_string_pretty(product).unwrap_or_else(|_| "{}".to_string())
    }

    fn json_products(&self, products: &[Product]) -> String {
        serde_json::to_string_pretty(products).unwrap_or_else(|_| "[]".to_string())
    }

    // Table formatting

    fn table_single(&self, product: &Product) -> String {
        let mut lines = Vec::new();

        lines.push(format!("ASIN:    {}", product.asin));
        lines.push(format!("Title:   {}", product.title));
        lines.push(format!("URL:     {}", product.url));

        if let Some(price) = &product.price {
            if price.is_hidden {
                lines.push("Price:   See price in cart".to_string());
            } else {
                let price_str = if let Some(orig) = price.original {
                    format!("{} {:.2} (was {:.2})", price.currency, price.current, orig)
                } else {
                    format!("{} {:.2}", price.currency, price.current)
                };
                lines.push(format!("Price:   {}", price_str));
            }
        } else {
            lines.push("Price:   N/A".to_string());
        }

        if let Some(rating) = &product.rating {
            lines.push(format!("Rating:  {:.1}/5 ({} reviews)", rating.stars, rating.review_count));
        } else {
            lines.push("Rating:  N/A".to_string());
        }

        let mut badges = Vec::new();
        if product.is_prime {
            badges.push("Prime");
        }
        if product.is_amazon_choice {
            badges.push("Amazon's Choice");
        }
        if product.is_sponsored {
            badges.push("Sponsored");
        }
        if !badges.is_empty() {
            lines.push(format!("Badges:  {}", badges.join(", ")));
        }

        if let Some(brand) = &product.brand {
            lines.push(format!("Brand:   {}", brand));
        }

        lines.push(format!(
            "Stock:   {}",
            if product.in_stock { "In Stock" } else { "Out of Stock" }
        ));

        lines.join("\n")
    }

    fn table_products(&self, products: &[Product]) -> String {
        // Calculate column widths
        let asin_width = 10;
        let price_width = 12;
        let rating_width = 8;
        let prime_width = 5;
        let title_width = 50;

        let mut lines = Vec::new();

        // Header
        lines.push(format!(
            "{:<asin_width$}  {:<price_width$}  {:<rating_width$}  {:<prime_width$}  {}",
            "ASIN", "Price", "Rating", "Prime", "Title"
        ));
        lines.push(format!(
            "{:-<asin_width$}  {:-<price_width$}  {:-<rating_width$}  {:-<prime_width$}  {:-<title_width$}",
            "", "", "", "", ""
        ));

        // Rows
        for product in products {
            let price_str = match &product.price {
                Some(p) if !p.is_hidden => format!("{:.2}", p.current),
                Some(_) => "In cart".to_string(),
                None => "N/A".to_string(),
            };

            let rating_str = match &product.rating {
                Some(r) => format!("{:.1}", r.stars),
                None => "N/A".to_string(),
            };

            let prime_str = if product.is_prime { "Yes" } else { "No" };

            let title = if product.title.len() > title_width {
                format!("{}...", &product.title[..title_width - 3])
            } else {
                product.title.clone()
            };

            lines.push(format!(
                "{:<asin_width$}  {:>price_width$}  {:>rating_width$}  {:<prime_width$}  {}",
                product.asin, price_str, rating_str, prime_str, title
            ));
        }

        lines.push(String::new());
        lines.push(format!("Total: {} products", products.len()));

        lines.join("\n")
    }

    // Markdown formatting

    fn markdown_single(&self, product: &Product) -> String {
        let mut lines = Vec::new();

        lines.push(format!("## {}", product.title));
        lines.push(String::new());

        lines.push(format!("- **ASIN:** {}", product.asin));
        lines.push(format!("- **URL:** [View on Amazon]({})", product.url));

        if let Some(price) = &product.price {
            if price.is_hidden {
                lines.push("- **Price:** See price in cart".to_string());
            } else if let Some(orig) = price.original {
                lines.push(format!(
                    "- **Price:** {} {:.2} ~~{:.2}~~",
                    price.currency, price.current, orig
                ));
            } else {
                lines.push(format!("- **Price:** {} {:.2}", price.currency, price.current));
            }
        }

        if let Some(rating) = &product.rating {
            lines.push(format!(
                "- **Rating:** {:.1}/5 ({} reviews)",
                rating.stars, rating.review_count
            ));
        }

        if let Some(brand) = &product.brand {
            lines.push(format!("- **Brand:** {}", brand));
        }

        let mut badges = Vec::new();
        if product.is_prime {
            badges.push("✓ Prime");
        }
        if product.is_amazon_choice {
            badges.push("⭐ Amazon's Choice");
        }
        if !badges.is_empty() {
            lines.push(format!("- **Badges:** {}", badges.join(", ")));
        }

        lines.join("\n")
    }

    fn markdown_products(&self, products: &[Product]) -> String {
        let mut lines = Vec::new();

        lines.push("| ASIN | Price | Rating | Prime | Title |".to_string());
        lines.push("|------|-------|--------|-------|-------|".to_string());

        for product in products {
            let price_str = match &product.price {
                Some(p) if !p.is_hidden => format!("{:.2}", p.current),
                Some(_) => "In cart".to_string(),
                None => "N/A".to_string(),
            };

            let rating_str = match &product.rating {
                Some(r) => format!("{:.1}", r.stars),
                None => "N/A".to_string(),
            };

            let prime_str = if product.is_prime { "✓" } else { "" };

            let title = if product.title.len() > 40 {
                format!("{}...", &product.title[..37])
            } else {
                product.title.clone()
            };

            lines.push(format!(
                "| {} | {} | {} | {} | [{}]({}) |",
                product.asin, price_str, rating_str, prime_str, title, product.url
            ));
        }

        lines.push(String::new());
        lines.push(format!("*{} products found*", products.len()));

        lines.join("\n")
    }

    // CSV formatting

    fn csv_header(&self) -> String {
        "asin,title,price,original_price,currency,rating,reviews,prime,sponsored,amazon_choice,in_stock,brand,url"
            .to_string()
    }

    fn csv_products(&self, products: &[Product]) -> String {
        let mut lines = Vec::new();
        lines.push(self.csv_header());

        for product in products {
            let price = product
                .price
                .as_ref()
                .map(|p| if p.is_hidden { String::new() } else { p.current.to_string() })
                .unwrap_or_default();

            let original = product
                .price
                .as_ref()
                .and_then(|p| p.original.map(|o| o.to_string()))
                .unwrap_or_default();

            let currency = product.price.as_ref().map(|p| p.currency.clone()).unwrap_or_default();

            let rating = product.rating.as_ref().map(|r| r.stars.to_string()).unwrap_or_default();

            let reviews =
                product.rating.as_ref().map(|r| r.review_count.to_string()).unwrap_or_default();

            let title = Self::csv_escape(&product.title);
            let brand = product.brand.as_ref().map(|b| Self::csv_escape(b)).unwrap_or_default();

            lines.push(format!(
                "{},{},{},{},{},{},{},{},{},{},{},{},{}",
                product.asin,
                title,
                price,
                original,
                currency,
                rating,
                reviews,
                product.is_prime,
                product.is_sponsored,
                product.is_amazon_choice,
                product.in_stock,
                brand,
                product.url
            ));
        }

        lines.join("\n")
    }

    fn csv_escape(s: &str) -> String {
        if s.contains(',') || s.contains('"') || s.contains('\n') {
            format!("\"{}\"", s.replace('"', "\"\""))
        } else {
            s.to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::amazon::models::{Price, Rating};

    fn make_product() -> Product {
        Product {
            asin: "B08N5WRWNW".to_string(),
            title: "Test Product Title".to_string(),
            url: "https://amazon.com/dp/B08N5WRWNW".to_string(),
            image_url: Some("https://images.amazon.com/test.jpg".to_string()),
            price: Some(Price::with_discount(29.99, 39.99, "USD")),
            rating: Some(Rating::new(4.5, 1234)),
            is_sponsored: false,
            is_prime: true,
            is_amazon_choice: true,
            in_stock: true,
            brand: Some("TestBrand".to_string()),
        }
    }

    fn make_minimal_product() -> Product {
        Product {
            asin: "MINIMAL123".to_string(),
            title: "Minimal Product".to_string(),
            url: "https://amazon.com/dp/MINIMAL123".to_string(),
            image_url: None,
            price: None,
            rating: None,
            is_sponsored: false,
            is_prime: false,
            is_amazon_choice: false,
            in_stock: false,
            brand: None,
        }
    }

    fn make_sponsored_product() -> Product {
        Product {
            asin: "SPONSORED1".to_string(),
            title: "Sponsored Product".to_string(),
            url: "https://amazon.com/dp/SPONSORED1".to_string(),
            image_url: None,
            price: Some(Price::simple(19.99, "USD")),
            rating: Some(Rating::new(3.5, 50)),
            is_sponsored: true,
            is_prime: false,
            is_amazon_choice: false,
            in_stock: true,
            brand: None,
        }
    }

    fn make_hidden_price_product() -> Product {
        Product {
            asin: "HIDDEN1234".to_string(),
            title: "Hidden Price Product".to_string(),
            url: "https://amazon.com/dp/HIDDEN1234".to_string(),
            image_url: None,
            price: Some(Price::hidden("USD")),
            rating: None,
            is_sponsored: false,
            is_prime: true,
            is_amazon_choice: false,
            in_stock: true,
            brand: None,
        }
    }

    fn make_long_title_product() -> Product {
        Product {
            asin: "LONGTITLE1".to_string(),
            title: "This is a very long product title that exceeds fifty characters and should be truncated in table output".to_string(),
            url: "https://amazon.com/dp/LONGTITLE1".to_string(),
            image_url: None,
            price: Some(Price::simple(49.99, "USD")),
            rating: Some(Rating::new(4.0, 500)),
            is_sponsored: false,
            is_prime: true,
            is_amazon_choice: false,
            in_stock: true,
            brand: Some("LongBrand".to_string()),
        }
    }

    // JSON format tests

    #[test]
    fn test_json_single_product() {
        let formatter = Formatter::new(OutputFormat::Json);
        let product = make_product();
        let output = formatter.format_product(&product);

        assert!(output.contains("B08N5WRWNW"));
        assert!(output.contains("Test Product Title"));
        assert!(output.contains("29.99"));
        assert!(output.contains("39.99"));
        assert!(output.contains("4.5"));
        assert!(output.contains("1234"));
        assert!(output.contains("TestBrand"));
    }

    #[test]
    fn test_json_multiple_products() {
        let formatter = Formatter::new(OutputFormat::Json);
        let products = vec![make_product(), make_minimal_product()];
        let output = formatter.format_products(&products);

        assert!(output.starts_with('['));
        assert!(output.ends_with(']'));
        assert!(output.contains("B08N5WRWNW"));
        assert!(output.contains("MINIMAL123"));
    }

    #[test]
    fn test_json_empty() {
        let formatter = Formatter::new(OutputFormat::Json);
        let output = formatter.format_products(&[]);
        assert_eq!(output, "[]");
    }

    // Table format tests

    #[test]
    fn test_table_single_product() {
        let formatter = Formatter::new(OutputFormat::Table);
        let product = make_product();
        let output = formatter.format_product(&product);

        assert!(output.contains("ASIN:    B08N5WRWNW"));
        assert!(output.contains("Title:   Test Product Title"));
        assert!(output.contains("URL:     https://amazon.com/dp/B08N5WRWNW"));
        assert!(output.contains("Price:   USD 29.99 (was 39.99)"));
        assert!(output.contains("Rating:  4.5/5 (1234 reviews)"));
        assert!(output.contains("Badges:  Prime, Amazon's Choice"));
        assert!(output.contains("Brand:   TestBrand"));
        assert!(output.contains("Stock:   In Stock"));
    }

    #[test]
    fn test_table_single_minimal_product() {
        let formatter = Formatter::new(OutputFormat::Table);
        let product = make_minimal_product();
        let output = formatter.format_product(&product);

        assert!(output.contains("ASIN:    MINIMAL123"));
        assert!(output.contains("Price:   N/A"));
        assert!(output.contains("Rating:  N/A"));
        assert!(!output.contains("Badges:"));
        assert!(!output.contains("Brand:"));
        assert!(output.contains("Stock:   Out of Stock"));
    }

    #[test]
    fn test_table_single_hidden_price() {
        let formatter = Formatter::new(OutputFormat::Table);
        let product = make_hidden_price_product();
        let output = formatter.format_product(&product);

        assert!(output.contains("Price:   See price in cart"));
        assert!(output.contains("Badges:  Prime"));
    }

    #[test]
    fn test_table_single_sponsored() {
        let formatter = Formatter::new(OutputFormat::Table);
        let product = make_sponsored_product();
        let output = formatter.format_product(&product);

        assert!(output.contains("Badges:  Sponsored"));
    }

    #[test]
    fn test_table_multiple_products() {
        let formatter = Formatter::new(OutputFormat::Table);
        let products = vec![make_product(), make_minimal_product(), make_sponsored_product()];
        let output = formatter.format_products(&products);

        // Header
        assert!(output.contains("ASIN"));
        assert!(output.contains("Price"));
        assert!(output.contains("Rating"));
        assert!(output.contains("Prime"));
        assert!(output.contains("Title"));

        // Separator line
        assert!(output.contains("----------"));

        // Products
        assert!(output.contains("B08N5WRWNW"));
        assert!(output.contains("MINIMAL123"));
        assert!(output.contains("SPONSORED1"));
        assert!(output.contains("29.99"));
        assert!(output.contains("N/A"));
        assert!(output.contains("Yes"));
        assert!(output.contains("No"));
        assert!(output.contains("Total: 3 products"));
    }

    #[test]
    fn test_table_long_title_truncation() {
        let formatter = Formatter::new(OutputFormat::Table);
        let products = vec![make_long_title_product()];
        let output = formatter.format_products(&products);

        assert!(output.contains("This is a very long product title that exceeds"));
        assert!(output.contains("..."));
    }

    #[test]
    fn test_table_hidden_price_in_list() {
        let formatter = Formatter::new(OutputFormat::Table);
        let products = vec![make_hidden_price_product()];
        let output = formatter.format_products(&products);

        assert!(output.contains("In cart"));
    }

    #[test]
    fn test_table_empty() {
        let formatter = Formatter::new(OutputFormat::Table);
        let output = formatter.format_products(&[]);
        assert_eq!(output, "No products found.");
    }

    // Markdown format tests

    #[test]
    fn test_markdown_single_product() {
        let formatter = Formatter::new(OutputFormat::Markdown);
        let product = make_product();
        let output = formatter.format_product(&product);

        assert!(output.contains("## Test Product Title"));
        assert!(output.contains("- **ASIN:** B08N5WRWNW"));
        assert!(output.contains("- **URL:** [View on Amazon](https://amazon.com/dp/B08N5WRWNW)"));
        assert!(output.contains("- **Price:** USD 29.99 ~~39.99~~"));
        assert!(output.contains("- **Rating:** 4.5/5 (1234 reviews)"));
        assert!(output.contains("- **Brand:** TestBrand"));
        assert!(output.contains("✓ Prime"));
        assert!(output.contains("⭐ Amazon's Choice"));
    }

    #[test]
    fn test_markdown_single_minimal() {
        let formatter = Formatter::new(OutputFormat::Markdown);
        let product = make_minimal_product();
        let output = formatter.format_product(&product);

        assert!(output.contains("## Minimal Product"));
        assert!(output.contains("- **ASIN:** MINIMAL123"));
        assert!(!output.contains("- **Price:**"));
        assert!(!output.contains("- **Rating:**"));
        assert!(!output.contains("- **Brand:**"));
        assert!(!output.contains("- **Badges:**"));
    }

    #[test]
    fn test_markdown_single_hidden_price() {
        let formatter = Formatter::new(OutputFormat::Markdown);
        let product = make_hidden_price_product();
        let output = formatter.format_product(&product);

        assert!(output.contains("- **Price:** See price in cart"));
    }

    #[test]
    fn test_markdown_single_simple_price() {
        let formatter = Formatter::new(OutputFormat::Markdown);
        let product = make_sponsored_product();
        let output = formatter.format_product(&product);

        assert!(output.contains("- **Price:** USD 19.99"));
        assert!(!output.contains("~~")); // No strikethrough for non-discounted
    }

    #[test]
    fn test_markdown_multiple_products() {
        let formatter = Formatter::new(OutputFormat::Markdown);
        let products = vec![make_product(), make_minimal_product()];
        let output = formatter.format_products(&products);

        // Table header
        assert!(output.contains("| ASIN | Price | Rating | Prime | Title |"));
        assert!(output.contains("|------|-------|--------|-------|-------|"));

        // Products
        assert!(output.contains("B08N5WRWNW"));
        assert!(output.contains("29.99"));
        assert!(output.contains("4.5"));
        assert!(output.contains("✓")); // Prime checkmark
        assert!(output.contains("MINIMAL123"));
        assert!(output.contains("N/A"));

        // Footer
        assert!(output.contains("*2 products found*"));
    }

    #[test]
    fn test_markdown_long_title_truncation() {
        let formatter = Formatter::new(OutputFormat::Markdown);
        let products = vec![make_long_title_product()];
        let output = formatter.format_products(&products);

        // Markdown truncates to 40 chars
        assert!(output.contains("..."));
    }

    #[test]
    fn test_markdown_empty() {
        let formatter = Formatter::new(OutputFormat::Markdown);
        let output = formatter.format_products(&[]);
        assert_eq!(output, "No products found.");
    }

    #[test]
    fn test_markdown_hidden_price_in_table() {
        let formatter = Formatter::new(OutputFormat::Markdown);
        let products = vec![make_hidden_price_product()];
        let output = formatter.format_products(&products);

        assert!(output.contains("In cart"));
    }

    // CSV format tests

    #[test]
    fn test_csv_header() {
        let formatter = Formatter::new(OutputFormat::Csv);
        let header = formatter.csv_header();
        assert_eq!(
            header,
            "asin,title,price,original_price,currency,rating,reviews,prime,sponsored,amazon_choice,in_stock,brand,url"
        );
    }

    #[test]
    fn test_csv_single_product() {
        let formatter = Formatter::new(OutputFormat::Csv);
        let product = make_product();
        let output = formatter.format_product(&product);

        let lines: Vec<&str> = output.lines().collect();
        assert_eq!(lines.len(), 2);
        assert!(lines[0].starts_with("asin,title,price"));
        assert!(lines[1].contains("B08N5WRWNW"));
        assert!(lines[1].contains("Test Product Title"));
        assert!(lines[1].contains("29.99"));
        assert!(lines[1].contains("39.99"));
        assert!(lines[1].contains("USD"));
        assert!(lines[1].contains("4.5"));
        assert!(lines[1].contains("1234"));
        assert!(lines[1].contains("true")); // is_prime
        assert!(lines[1].contains("false")); // is_sponsored
        assert!(lines[1].contains("TestBrand"));
    }

    #[test]
    fn test_csv_multiple_products() {
        let formatter = Formatter::new(OutputFormat::Csv);
        let products = vec![make_product(), make_minimal_product(), make_sponsored_product()];
        let output = formatter.format_products(&products);

        let lines: Vec<&str> = output.lines().collect();
        assert_eq!(lines.len(), 4); // Header + 3 products
        assert!(lines[1].contains("B08N5WRWNW"));
        assert!(lines[2].contains("MINIMAL123"));
        assert!(lines[3].contains("SPONSORED1"));
    }

    #[test]
    fn test_csv_hidden_price() {
        let formatter = Formatter::new(OutputFormat::Csv);
        let products = vec![make_hidden_price_product()];
        let output = formatter.format_products(&products);

        let lines: Vec<&str> = output.lines().collect();
        // Hidden price should result in empty price field
        assert!(lines[1].contains("HIDDEN1234"));
    }

    #[test]
    fn test_csv_empty() {
        let formatter = Formatter::new(OutputFormat::Csv);
        let output = formatter.format_products(&[]);
        assert_eq!(
            output,
            "asin,title,price,original_price,currency,rating,reviews,prime,sponsored,amazon_choice,in_stock,brand,url"
        );
    }

    #[test]
    fn test_csv_escape() {
        assert_eq!(Formatter::csv_escape("simple"), "simple");
        assert_eq!(Formatter::csv_escape("with,comma"), "\"with,comma\"");
        assert_eq!(Formatter::csv_escape("with\"quote"), "\"with\"\"quote\"");
        assert_eq!(Formatter::csv_escape("with\nnewline"), "\"with\nnewline\"");
        assert_eq!(Formatter::csv_escape("combo,\"test\"\n"), "\"combo,\"\"test\"\"\n\"");
    }

    #[test]
    fn test_csv_escape_product_with_special_chars() {
        let formatter = Formatter::new(OutputFormat::Csv);
        let mut product = make_product();
        product.title = "Product, with \"quotes\" and\nnewlines".to_string();
        product.brand = Some("Brand, Inc.".to_string());

        let output = formatter.format_product(&product);
        // Should have escaped fields
        assert!(output.contains("\"Product, with \"\"quotes\"\" and\nnewlines\""));
        assert!(output.contains("\"Brand, Inc.\""));
    }

    // Edge case tests

    #[test]
    fn test_format_product_all_formats() {
        let product = make_product();

        // All formats should work without panicking
        let json = Formatter::new(OutputFormat::Json).format_product(&product);
        let table = Formatter::new(OutputFormat::Table).format_product(&product);
        let md = Formatter::new(OutputFormat::Markdown).format_product(&product);
        let csv = Formatter::new(OutputFormat::Csv).format_product(&product);

        assert!(!json.is_empty());
        assert!(!table.is_empty());
        assert!(!md.is_empty());
        assert!(!csv.is_empty());
    }

    #[test]
    fn test_format_products_all_formats() {
        let products = vec![make_product(), make_minimal_product()];

        let json = Formatter::new(OutputFormat::Json).format_products(&products);
        let table = Formatter::new(OutputFormat::Table).format_products(&products);
        let md = Formatter::new(OutputFormat::Markdown).format_products(&products);
        let csv = Formatter::new(OutputFormat::Csv).format_products(&products);

        assert!(!json.is_empty());
        assert!(!table.is_empty());
        assert!(!md.is_empty());
        assert!(!csv.is_empty());
    }
}
