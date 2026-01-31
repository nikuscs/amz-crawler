# CLAUDE.md - Project Guidelines

## Project Overview
amz-crawler - A fast, stateless Amazon product search CLI with TLS fingerprint emulation.

## Tech Stack
- **Language:** Rust (edition 2021)
- **Async Runtime:** Tokio
- **HTTP Client:** wreq (reqwest fork with TLS fingerprint emulation)
- **HTML Parsing:** scraper (based on html5ever)
- **CLI:** Clap with derive macros
- **Logging:** Tracing

## Architecture

```
src/
├── main.rs              # CLI entry point
├── lib.rs               # Library exports
├── config.rs            # TOML + env + CLI config
├── amazon/              # Amazon-specific modules
│   ├── mod.rs           # Exports
│   ├── client.rs        # HTTP client (wreq with Chrome emulation)
│   ├── parser.rs        # HTML parsing
│   ├── selectors.rs     # CSS selectors (standalone for easy updates)
│   ├── models.rs        # Product, Price, Rating structs
│   └── regions.rs       # Amazon TLDs (15 regions)
├── filters/             # Extensible filter system
│   ├── mod.rs           # Filter trait + FilterChain
│   ├── price.rs         # Price range filter
│   ├── rating.rs        # Minimum rating filter
│   ├── keyword.rs       # Title keyword filter
│   └── prime.rs         # Prime-only filter
├── commands/            # CLI command handlers
│   ├── mod.rs           # Exports
│   ├── search.rs        # Search command
│   ├── product.rs       # ASIN lookup command
│   └── compare.rs       # TropicalPrice commands (feature: tropical)
├── format/              # Output formatting
│   └── mod.rs           # Table/JSON/Markdown/CSV formatters
└── tropical/            # TropicalPrice EU comparison (feature: tropical)
    ├── mod.rs           # Exports
    ├── client.rs        # TropicalPrice HTTP client
    ├── models.rs        # PriceComparison, CountryPrice
    └── parser.rs        # HTML parsing for TropicalPrice

tests/
├── fixtures/            # HTML fixtures for parser tests
└── parser_integration.rs # Integration tests
```

## Feature Flags

| Feature | Description | Dependencies |
|---------|-------------|--------------|
| `tropical` | TropicalPrice EU price comparison | regex-lite |

Build with features:
```bash
cargo build --features tropical
cargo build --all-features
```

## Code Standards

### Formatting
- Use `cargo fmt` before committing
- Max line width: 100 chars (see rustfmt.toml)
- Edition 2021 style

### Error Handling
- Use `anyhow::Result` for application errors
- Use `thiserror` for library error types
- Always provide context with `.context()` or `.with_context()`

### Async
- Multi-threaded Tokio runtime
- Use `tokio::time::sleep` for delays

### Testing
- Unit tests in same file with `#[cfg(test)]` module
- Integration tests in `tests/` directory
- Use HTML fixtures for parser tests
- Run `cargo test` and `cargo test --all-features` before committing

## Common Commands

```bash
# Development
cargo build                           # Debug build
cargo build --release                 # Release build
cargo build --features tropical       # With TropicalPrice support

# Quality Checks (run these before committing)
cargo fmt                             # Format code
cargo clippy --all-targets            # Run linter
cargo clippy --all-features --all-targets  # With all features
cargo test                            # Run all tests
cargo test --all-features             # Test all features

# Running
./target/release/amz-crawler --help
./target/release/amz-crawler search "rust book" --max 5
./target/release/amz-crawler product B08N5WRWNW
./target/release/amz-crawler compare B08N5WRWNW   # (requires tropical feature)
./target/release/amz-crawler tropical "iphone 15" # (requires tropical feature)
```

## End-of-Task Workflow

**ALWAYS run these commands at the end of ANY significant task:**

```bash
# 1. Format code
cargo fmt

# 2. Check lints (must have 0 warnings for CI)
cargo clippy --all-targets
cargo clippy --all-features --all-targets

# 3. Run tests (must all pass)
cargo test
cargo test --all-features
```

**Why this matters:**
- CI runs with `RUSTFLAGS="-Dwarnings"` which treats warnings as errors
- Formatting issues will fail the CI format check
- Broken tests block merging to main

## Adding New Features

### New Filter
1. Create `src/filters/my_filter.rs`
2. Implement the `Filter` trait
3. Export in `src/filters/mod.rs`
4. Add to `FilterChainBuilder` if needed

### New Output Format
1. Add variant to `OutputFormat` enum in `config.rs`
2. Implement format method in `src/format/mod.rs`

### New Region
1. Add variant to `Region` enum in `src/amazon/regions.rs`
2. Implement `domain()`, `currency()`, `accept_language()` methods

### Updating Selectors
When Amazon changes their HTML structure:
1. Capture sample HTML in `tests/fixtures/`
2. Update selectors in `src/amazon/selectors.rs`
3. Add/update tests to verify parsing works

### Adding Optional Features
1. Add feature to `[features]` in `Cargo.toml`
2. Use `#[cfg(feature = "feature_name")]` for conditional compilation
3. Add optional dependencies with `optional = true`
4. Document in this file and README.md

## Configuration

- Config file: `config.toml` (see `config.example.toml`)
- Environment variables: `AMZ_REGION`, `AMZ_PROXY`, `AMZ_DELAY`
- CLI flags override config file and env vars

## Anti-Bot Strategy

The project uses wreq instead of reqwest for TLS fingerprint emulation:
- Browser impersonation (Chrome 131)
- JA3/JA4 fingerprint spoofing
- HTTP/2 settings matching real browsers
- Random delays with jitter (2-5s default)
- Cookie persistence

## Dependencies Policy

- Prefer well-maintained crates
- wreq for HTTP (TLS fingerprinting is critical)
- Use `optional = true` for feature-gated dependencies
- Keep deps up to date: `cargo upgrade`
- Check for security issues: `cargo audit`
