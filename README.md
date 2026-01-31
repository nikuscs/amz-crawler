# 🦎 amz-crawler

![CI](https://github.com/nikuscs/amz-crawler/actions/workflows/ci.yml/badge.svg)
![Release](https://img.shields.io/github/v/release/nikuscs/amz-crawler)
![License](https://img.shields.io/badge/license-MIT-blue.svg)

**Fast Rust CLI to search Amazon products with browser-grade TLS fingerprinting.**

> **Disclaimer:** This project is for **educational purposes and AI automation research only**.
> The authors are not responsible for any misuse or for any damages resulting from the use of this tool.
> Users are solely responsible for ensuring compliance with applicable laws and the terms of service
> of any websites accessed. This software is provided "as-is" without warranty of any kind.
>
> If you are a rights holder and wish to have this project removed, please [contact me](https://github.com/nikuscs).

> **Note:** This project was partially developed with AI assistance and may contain bugs or unexpected behavior. Use at your own risk.

Search products, filter by price/rating, compare prices across 15 regions, and find the cheapest EU deals with TropicalPrice integration.

## Why?

- **Stealth** — Chrome 131 TLS fingerprint via [wreq](https://github.com/pwnwriter/wreq). Bypasses basic bot detection.
- **Fast** — Native Rust. No browser overhead.
- **EU Price Comparison** — Find the cheapest Amazon store across ES/DE/FR/IT/UK/NL with TropicalPrice.
- **Flexible Output** — Table, JSON, Markdown, CSV. Pipe to `jq`, feed to scripts, or read in terminal.

## Install

```bash
# From source (requires Rust)
cargo install --git https://github.com/nikuscs/amz-crawler --features tropical

# Or clone and build
git clone https://github.com/nikuscs/amz-crawler
cd amz-crawler
cargo build --release --features tropical
```

Pre-built binaries available in [Releases](https://github.com/nikuscs/amz-crawler/releases).

## Usage

### Search Amazon

```bash
amz-crawler search "mechanical keyboard"
amz-crawler search "laptop" --max 10 --min-price 500 --max-price 1000
amz-crawler search "headphones" --min-rating 4.5 --prime-only --no-sponsored
amz-crawler search "monitor" --keywords ips,4k --exclude refurbished
amz-crawler --region de search "kaffeemaschine"
```

### Product Details

```bash
amz-crawler product B08N5WRWNW
amz-crawler product B08N5WRWNW B09HMZ6S1Y B0BSHF7WHW  # Multiple ASINs
```

### EU Price Comparison (TropicalPrice)

Find the cheapest price across EU Amazon stores:

```bash
# Search TropicalPrice catalog
amz-crawler tropical "sony wh-1000xm5" --max 5

# Compare specific product across EU stores
amz-crawler compare B0C8PSMPTH
```

**Output:**
```
📦 Sony WH-1000XM5 Wireless Headphones

Best at 🇩🇪 DE: €279.99
🛒 https://www.amazon.de/dp/B0C8PSMPTH

🏆🇩🇪 DE: €279.99
  🇪🇸 ES: €329.99 (+€50, +18%)
  🇫🇷 FR: €339.99 (+€60, +21%)
  🇮🇹 IT: €349.99 (+€70, +25%)

💰 Max savings: €70.00 (25%)

🔗 Links:
   🇩🇪 DE: https://www.amazon.de/dp/B0C8PSMPTH
   🇪🇸 ES: https://www.amazon.es/dp/B0C8PSMPTH
   🇫🇷 FR: https://www.amazon.fr/dp/B0C8PSMPTH
   🇮🇹 IT: https://www.amazon.it/dp/B0C8PSMPTH
```

### Regions

```bash
amz-crawler regions  # List all supported regions
```

**Supported:** `us` `uk` `de` `fr` `es` `it` `ca` `au` `jp` `in` `br` `mx` `nl` `se` `pl`

## Options

### Search Filters

| Flag | Description |
|------|-------------|
| `--max` | Max results (default: 20) |
| `--min-price` | Minimum price |
| `--max-price` | Maximum price |
| `--min-rating` | Minimum rating (1.0-5.0) |
| `--prime-only` | Only Prime-eligible |
| `--no-sponsored` | Exclude sponsored listings |
| `--keywords` | Required keywords in title (comma-separated) |
| `--exclude` | Exclude keywords from title (comma-separated) |

### Global Options

| Flag | Description |
|------|-------------|
| `--region` | Amazon region (default: us) |
| `--format` | Output: table, json, markdown, csv |
| `--proxy` | Proxy URL (socks5/http) |
| `--delay` | Request delay in ms (default: 2000) |
| `--config` | Config file path |

## Configuration

Create `~/.config/amz-crawler/config.toml`:

```toml
region = "es"
delay_ms = 2000
delay_jitter_ms = 3000
max_results = 20
format = "table"
# proxy = "socks5://127.0.0.1:1080"
```

Environment variables: `AMZ_REGION`, `AMZ_PROXY`, `AMZ_DELAY`

## Output Formats

```bash
amz-crawler search "laptop" --format json      # JSON (for scripts)
amz-crawler search "laptop" --format markdown  # Markdown (for LLMs)
amz-crawler search "laptop" --format csv       # CSV (for spreadsheets)
amz-crawler search "laptop" --format table     # Table (default)
```

## How It Works

1. **TLS Fingerprinting** — Uses [wreq](https://github.com/pwnwriter/wreq) to emulate Chrome 131 TLS handshake (JA3/JA4).
2. **Full Browser Headers** — Sends complete header set including Sec-Fetch-*, cookies, etc.
3. **Request Jitter** — Random delays (2-5s default) to appear human.
4. **Smart Parsing** — Handles regional price formats (1.234,56 € vs $1,234.56).

## Related

- [TropicalPrice](https://tropicalprice.com) — EU Amazon price comparison service
- [wreq](https://github.com/pwnwriter/wreq) — Rust HTTP client with TLS fingerprinting

## License

MIT
