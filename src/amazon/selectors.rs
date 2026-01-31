//! CSS selectors for Amazon HTML parsing.
//!
//! This file contains all CSS selectors used for parsing Amazon pages.
//! Update this file when Amazon changes their HTML structure.
//!
//! **Update process**: When parsing fails, capture HTML sample,
//! update selectors, and add test fixture.

use scraper::Selector;
use std::sync::LazyLock;

/// Selectors for search results pages.
pub mod search {
    use super::*;

    /// Product card container - main search result item.
    pub static RESULT: LazyLock<Selector> =
        LazyLock::new(|| Selector::parse("[data-component-type='s-search-result']").unwrap());

    /// ASIN attribute on result card.
    pub static ASIN_ATTR: &str = "data-asin";

    /// Product title text.
    pub static TITLE: LazyLock<Selector> = LazyLock::new(|| {
        Selector::parse(
            "h2 a span, \
             h2 span.a-text-normal, \
             .a-size-medium.a-text-normal, \
             .a-size-base-plus.a-text-normal",
        )
        .unwrap()
    });

    /// Title link for URL extraction.
    pub static TITLE_LINK: LazyLock<Selector> = LazyLock::new(|| {
        Selector::parse(
            "h2 a.a-link-normal, \
             h2 a.s-link-style, \
             .a-link-normal.s-underline-text",
        )
        .unwrap()
    });

    /// Product image.
    pub static IMAGE: LazyLock<Selector> = LazyLock::new(|| {
        Selector::parse(
            "img.s-image, \
             .s-product-image-container img",
        )
        .unwrap()
    });

    /// Whole price (dollars/euros part).
    pub static PRICE_WHOLE: LazyLock<Selector> = LazyLock::new(|| {
        Selector::parse(
            ".a-price .a-price-whole, \
             .a-price-whole",
        )
        .unwrap()
    });

    /// Fractional price (cents part).
    pub static PRICE_FRACTION: LazyLock<Selector> = LazyLock::new(|| {
        Selector::parse(
            ".a-price .a-price-fraction, \
             .a-price-fraction",
        )
        .unwrap()
    });

    /// Price symbol.
    pub static PRICE_SYMBOL: LazyLock<Selector> = LazyLock::new(|| {
        Selector::parse(
            ".a-price .a-price-symbol, \
             .a-price-symbol",
        )
        .unwrap()
    });

    /// Full price container (for current price).
    pub static PRICE_CURRENT: LazyLock<Selector> = LazyLock::new(|| {
        Selector::parse(
            ".a-price:not([data-a-strike]) .a-offscreen, \
             .a-price .a-offscreen",
        )
        .unwrap()
    });

    /// Original price (strikethrough).
    pub static PRICE_ORIGINAL: LazyLock<Selector> = LazyLock::new(|| {
        Selector::parse(
            ".a-price[data-a-strike] .a-offscreen, \
             .a-text-price .a-offscreen, \
             span[data-a-strike='true'] .a-offscreen",
        )
        .unwrap()
    });

    /// Price range container.
    pub static PRICE_RANGE: LazyLock<Selector> =
        LazyLock::new(|| Selector::parse(".a-price-range, .a-price + .a-price").unwrap());

    /// "See price in cart" text.
    pub static PRICE_HIDDEN: LazyLock<Selector> = LazyLock::new(|| {
        Selector::parse(
            ".a-color-base:contains('See price'), \
             .a-button-text:contains('cart')",
        )
        .unwrap_or_else(|_| Selector::parse(".a-color-base").unwrap())
    });

    /// Star rating element.
    pub static RATING_STARS: LazyLock<Selector> = LazyLock::new(|| {
        Selector::parse(
            "i.a-icon-star-small span.a-icon-alt, \
             i.a-icon-star span.a-icon-alt, \
             span.a-icon-alt",
        )
        .unwrap()
    });

    /// Review count link.
    pub static RATING_COUNT: LazyLock<Selector> = LazyLock::new(|| {
        Selector::parse(
            "span.a-size-base.s-underline-text, \
             a[href*='customerReviews'] span, \
             .a-size-base.puis-light-weight-text",
        )
        .unwrap()
    });

    /// Prime badge.
    pub static PRIME_BADGE: LazyLock<Selector> = LazyLock::new(|| {
        Selector::parse(
            "i.a-icon-prime, \
             .a-icon-prime, \
             [data-component-type='s-prime-badge']",
        )
        .unwrap()
    });

    /// Sponsored label.
    pub static SPONSORED: LazyLock<Selector> = LazyLock::new(|| {
        Selector::parse(
            ".puis-label-popover-default, \
             .s-label-popover-default, \
             span:contains('Sponsored'), \
             .a-color-secondary:contains('Sponsored')",
        )
        .unwrap_or_else(|_| Selector::parse(".puis-label-popover-default").unwrap())
    });

    /// Amazon's Choice badge.
    pub static AMAZON_CHOICE: LazyLock<Selector> = LazyLock::new(|| {
        Selector::parse(
            ".a-badge-text:contains('Choice'), \
             [data-component-type='s-merchandised-badge']",
        )
        .unwrap_or_else(|_| Selector::parse(".a-badge-text").unwrap())
    });

    /// Brand name.
    pub static BRAND: LazyLock<Selector> = LazyLock::new(|| {
        Selector::parse(
            ".a-size-base-plus.a-color-base, \
             .a-row.a-size-base.a-color-secondary span, \
             h5.s-line-clamp-1 span",
        )
        .unwrap()
    });

    /// "In stock" / availability indicator.
    pub static IN_STOCK: LazyLock<Selector> = LazyLock::new(|| {
        Selector::parse(
            ".a-color-success, \
             .a-color-price:contains('stock')",
        )
        .unwrap_or_else(|_| Selector::parse(".a-color-success").unwrap())
    });

    /// Total results count on page.
    pub static TOTAL_RESULTS: LazyLock<Selector> = LazyLock::new(|| {
        Selector::parse(
            ".a-section.a-spacing-small span:first-child, \
             .sg-col-inner .a-section span",
        )
        .unwrap()
    });

    /// Next page link.
    pub static NEXT_PAGE: LazyLock<Selector> = LazyLock::new(|| {
        Selector::parse(
            "a.s-pagination-next, \
             .s-pagination-item.s-pagination-next",
        )
        .unwrap()
    });
}

/// Selectors for individual product pages (ASIN lookup).
pub mod product {
    use super::*;

    /// Product title on detail page.
    pub static TITLE: LazyLock<Selector> = LazyLock::new(|| {
        Selector::parse(
            "#productTitle, \
             #title span, \
             .product-title-word-break",
        )
        .unwrap()
    });

    /// Current price on detail page.
    pub static PRICE: LazyLock<Selector> = LazyLock::new(|| {
        Selector::parse(
            "#corePrice_feature_div .a-price .a-offscreen, \
             #priceblock_ourprice, \
             #priceblock_dealprice, \
             .a-price .a-offscreen",
        )
        .unwrap()
    });

    /// Original price (before discount).
    pub static PRICE_ORIGINAL: LazyLock<Selector> = LazyLock::new(|| {
        Selector::parse(
            "#corePrice_feature_div .a-text-price .a-offscreen, \
             #priceblock_saleprice, \
             .a-text-price .a-offscreen",
        )
        .unwrap()
    });

    /// Main product image.
    pub static IMAGE: LazyLock<Selector> = LazyLock::new(|| {
        Selector::parse(
            "#landingImage, \
             #imgTagWrapperId img, \
             #main-image",
        )
        .unwrap()
    });

    /// Rating section.
    pub static RATING: LazyLock<Selector> = LazyLock::new(|| {
        Selector::parse(
            "#acrPopover span.a-icon-alt, \
             .a-icon-star span.a-icon-alt",
        )
        .unwrap()
    });

    /// Review count on detail page.
    pub static REVIEW_COUNT: LazyLock<Selector> = LazyLock::new(|| {
        Selector::parse(
            "#acrCustomerReviewText, \
             #acrCustomerReviewLink span",
        )
        .unwrap()
    });

    /// Brand/manufacturer.
    pub static BRAND: LazyLock<Selector> = LazyLock::new(|| {
        Selector::parse(
            "#bylineInfo, \
             .po-brand .po-break-word, \
             a#bylineInfo",
        )
        .unwrap()
    });

    /// Availability text.
    pub static AVAILABILITY: LazyLock<Selector> = LazyLock::new(|| {
        Selector::parse(
            "#availability span, \
             #outOfStock span, \
             .a-color-success",
        )
        .unwrap()
    });

    /// Prime badge on detail page.
    pub static PRIME: LazyLock<Selector> = LazyLock::new(|| {
        Selector::parse(
            "#prime-badge, \
             .a-icon-prime, \
             i.a-icon-prime",
        )
        .unwrap()
    });

    /// Amazon's Choice badge.
    pub static AMAZON_CHOICE: LazyLock<Selector> = LazyLock::new(|| {
        Selector::parse(
            "#acBadge_feature_div .a-badge-text, \
             .ac-badge-wrapper",
        )
        .unwrap()
    });

    /// ASIN from page (backup extraction).
    pub static ASIN: LazyLock<Selector> = LazyLock::new(|| {
        Selector::parse(
            "input[name='ASIN'], \
             th:contains('ASIN') + td",
        )
        .unwrap_or_else(|_| Selector::parse("input[name='ASIN']").unwrap())
    });
}

/// Selectors for detecting error/captcha pages.
pub mod errors {
    use super::*;

    /// CAPTCHA form.
    pub static CAPTCHA: LazyLock<Selector> = LazyLock::new(|| {
        Selector::parse(
            "form[action*='validateCaptcha'], \
             img[src*='captcha'], \
             .a-box-inner h4:contains('robot')",
        )
        .unwrap_or_else(|_| Selector::parse("form[action*='validateCaptcha']").unwrap())
    });

    /// "No results" message.
    pub static NO_RESULTS: LazyLock<Selector> = LazyLock::new(|| {
        Selector::parse(
            ".a-section.a-text-center.s-no-search-results, \
             span:contains('No results for')",
        )
        .unwrap_or_else(|_| Selector::parse(".s-no-search-results").unwrap())
    });

    /// Dog page (Amazon's error page).
    pub static DOG_PAGE: LazyLock<Selector> = LazyLock::new(|| {
        Selector::parse(
            "img[alt*='dog'], \
             .a-box-inner a[href='/ref=cs_503_link']",
        )
        .unwrap_or_else(|_| Selector::parse("img[alt*='dog']").unwrap())
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use scraper::Html;

    #[test]
    fn test_selectors_compile() {
        // Force evaluation of all lazy selectors to ensure they compile
        let _ = &*search::RESULT;
        let _ = &*search::TITLE;
        let _ = &*search::TITLE_LINK;
        let _ = &*search::PRICE_CURRENT;
        let _ = &*search::RATING_STARS;
        let _ = &*product::TITLE;
        let _ = &*product::PRICE;
        let _ = &*errors::CAPTCHA;
    }

    #[test]
    fn test_basic_selector_matching() {
        let html = Html::parse_document(
            r#"<div data-component-type="s-search-result" data-asin="B123">
                <h2><a class="a-link-normal" href="/dp/B123"><span>Test Product</span></a></h2>
            </div>"#,
        );

        let results: Vec<_> = html.select(&search::RESULT).collect();
        assert_eq!(results.len(), 1);

        let asin = results[0].value().attr(search::ASIN_ATTR);
        assert_eq!(asin, Some("B123"));
    }
}
