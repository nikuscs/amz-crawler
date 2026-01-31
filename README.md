# Amazon Product Search

Fast Rust CLI to search Amazon products with TLS fingerprint emulation.

## Install

```bash
cargo build --release

# With TropicalPrice EU comparison (optional)
cargo build --release --features tropical
```

## Quick Search

```bash
amz-crawler search "wireless mouse"
amz-crawler search "laptop" --max 10 --min-price 500 --max-price 1000
amz-crawler search "keyboard" --min-rating 4.0 --prime-only
amz-crawler search "headphones" --no-sponsored --keywords bluetooth,wireless
amz-crawler search "monitor" --exclude refurbished,renewed
amz-crawler search "ps5" --format json
amz-crawler search "macbook" --format markdown
```

| Flag | Description |
|------|-------------|
| `--max` | Max results (default: 20) |
| `--min-price` | Minimum price filter |
| `--max-price` | Maximum price filter |
| `--min-rating` | Minimum star rating (1.0-5.0) |
| `--prime-only` | Only Prime-eligible products |
| `--no-sponsored` | Exclude sponsored listings |
| `--keywords` | Required keywords in title (comma-separated) |
| `--exclude` | Excluded keywords from title (comma-separated) |
| `--format` | table, json, markdown, csv |

## Product Lookup

```bash
amz-crawler product B08N5WRWNW
amz-crawler product B08N5WRWNW B09HMZ6S1Y B0BSHF7WHW
amz-crawler product B08N5WRWNW --format json
```

## EU Price Comparison (TropicalPrice)

Compare prices across EU Amazon stores. Requires `--features tropical`.

```bash
# Compare a product across EU stores
amz-crawler compare B08N5WRWNW

# Search TropicalPrice for EU products
amz-crawler tropical "iphone 15" --max 10

# JSON output for scripts/LLMs
amz-crawler compare B08N5WRWNW --format json
```

| Flag | Description |
|------|-------------|
| `--max` | Max search results (default: 10) |
| `--format` | table, json |

Example output:
```
Best at 🇩🇪 DE: €89.99
🛒 https://www.amazon.de/dp/B08N5WRWNW

🏆🇩🇪 DE: €89.99
  🇪🇸 ES: €94.99 (+€5, +6%)
  🇫🇷 FR: €99.99 (+€10, +11%)
  🇮🇹 IT: €102.99 (+€13, +14%) ⚠️
```

## Regions

```bash
amz-crawler --region uk search "tea kettle"
amz-crawler --region de search "kaffeemaschine"
amz-crawler regions  # list all regions
```

Supported: `us` `uk` `de` `fr` `es` `it` `ca` `au` `jp` `in` `br` `mx` `nl` `se` `pl`

## Proxy

```bash
amz-crawler --proxy "socks5://127.0.0.1:1080" search "laptop"
amz-crawler --proxy "http://user:pass@proxy.com:8080" search "phone"
```

## Request Delay

```bash
amz-crawler --delay 3000 search "laptop"  # 3 second delay between requests
```

## Global Flags

| Flag | Description |
|------|-------------|
| `--config` | Config file path |
| `--region` | Amazon region (default: us) |
| `--proxy` | Proxy URL (socks5/http) |
| `--delay` | Delay between requests in ms (default: 2000) |
| `--format` | Output format (default: table) |
| `--verbose` | Enable debug logging |

## Output Formats

| Format | Flag | Use Case |
|--------|------|----------|
| Table | `--format table` | CLI output (default) |
| JSON | `--format json` | APIs, scripts |
| Markdown | `--format markdown` | LLMs, documentation |
| CSV | `--format csv` | Spreadsheets, data analysis |

JSON output includes: `asin`, `title`, `price`, `original_price`, `currency`, `rating`, `review_count`, `is_prime`, `is_sponsored`, `is_amazon_choice`, `in_stock`, `brand`, `url`, `image_url`

## Examples

### Example 1: Find cheap wireless mice with good ratings

```bash
amz-crawler search "wireless mouse" \
  --max 20 \
  --min-price 10 \
  --max-price 50 \
  --min-rating 4.0 \
  --prime-only \
  --no-sponsored
```

### Example 2: Compare laptop prices across EU (with tropical feature)

```bash
# Search for a laptop on TropicalPrice
amz-crawler tropical "macbook air m3" --max 5

# Compare specific ASIN across EU stores
amz-crawler compare B0CX23V2ZK --format json
```

### Example 3: JSON output for API/LLM integration

```bash
# Get results as JSON
amz-crawler search "rust programming book" --max 5 --format json

# Get results as markdown (for LLMs)
amz-crawler search "rust programming book" --max 5 --format markdown
```

### Example 4: Search with proxy for IP rotation

```bash
amz-crawler \
  --proxy "socks5://127.0.0.1:1080" \
  --delay 3000 \
  search "gaming laptop" \
  --max 30 \
  --min-rating 4.5
```

### Example 5: UK region with specific keywords

```bash
amz-crawler --region uk search "electric kettle" \
  --keywords stainless,steel \
  --exclude plastic \
  --prime-only
```

## Configuration

Create `config.toml`:

```toml
region = "us"
# proxy = "socks5://127.0.0.1:1080"
delay_ms = 2000
delay_jitter_ms = 3000
max_results = 20
format = "table"
prime_only = false
no_sponsored = false
```

Environment variables:
- `AMZ_REGION` - Override region
- `AMZ_PROXY` - Proxy URL
- `AMZ_DELAY` - Request delay in ms

## Features

- TLS fingerprint emulation (Chrome 131 via wreq)
- 15 Amazon regions with proper localization
- Smart price parsing (handles EU formats like 1.234,56 €)
- Price range and rating filters
- Prime-only and sponsored exclusion
- Keyword filtering (include/exclude)
- Multiple output formats
- Proxy support (SOCKS5/HTTP)
- Random request jitter for human-like behavior
- EU price comparison via TropicalPrice (optional feature)

## Anti-Bot Measures

- Browser TLS fingerprinting (JA3/JA4)
- HTTP/2 settings matching real browsers
- Full browser header set (Sec-Fetch-*, etc.)
- Cookie persistence
- Random delays with jitter (2-5s default)

## License

MIT
