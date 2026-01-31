//! amz-crawler - Fast, stateless Amazon product search CLI
//!
//! A Rust implementation with TLS fingerprint emulation for reliable scraping.

use amz_crawler::amazon::regions::Region;
use amz_crawler::commands::{ProductCommand, SearchCommand};
use amz_crawler::config::{Config, OutputFormat};
use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use tracing::Level;
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(
    name = "amz-crawler",
    version,
    about = "Fast, stateless Amazon product search CLI",
    long_about = "A Rust port of amzSear with TLS fingerprint emulation for reliable Amazon product searching."
)]
struct Cli {
    /// Amazon region to search
    #[arg(short, long, default_value = "us", global = true)]
    region: Region,

    /// Proxy URL (e.g., socks5://host:port)
    #[arg(long, global = true, env = "AMZ_PROXY")]
    proxy: Option<String>,

    /// Delay between requests in milliseconds
    #[arg(long, default_value = "2000", global = true, env = "AMZ_DELAY")]
    delay: u64,

    /// Path to config file
    #[arg(short, long, global = true)]
    config: Option<PathBuf>,

    /// Output format
    #[arg(short, long, default_value = "table", global = true)]
    format: OutputFormat,

    /// Enable verbose logging
    #[arg(short, long, global = true)]
    verbose: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Search for products
    #[command(alias = "s")]
    Search {
        /// Search query
        query: String,

        /// Maximum number of results
        #[arg(short, long, default_value = "20")]
        max: usize,

        /// Minimum price filter
        #[arg(long)]
        min_price: Option<f64>,

        /// Maximum price filter
        #[arg(long)]
        max_price: Option<f64>,

        /// Minimum rating filter (1.0-5.0)
        #[arg(long)]
        min_rating: Option<f32>,

        /// Only show Prime-eligible products
        #[arg(long)]
        prime_only: bool,

        /// Exclude sponsored products
        #[arg(long)]
        no_sponsored: bool,

        /// Required keywords in title (comma-separated)
        #[arg(long, value_delimiter = ',')]
        keywords: Option<Vec<String>>,

        /// Excluded keywords from title (comma-separated)
        #[arg(long, value_delimiter = ',')]
        exclude: Option<Vec<String>>,
    },

    /// Look up a product by ASIN
    #[command(alias = "p")]
    Product {
        /// ASIN(s) to look up
        #[arg(required = true)]
        asins: Vec<String>,
    },

    /// List supported regions
    Regions,

    /// Compare prices across EU Amazon stores (TropicalPrice)
    #[cfg(feature = "tropical")]
    #[command(alias = "c")]
    Compare {
        /// ASIN to compare
        asin: String,
    },

    /// Search TropicalPrice for EU products
    #[cfg(feature = "tropical")]
    Tropical {
        /// Search query
        query: String,

        /// Maximum number of results
        #[arg(short, long, default_value = "10")]
        max: usize,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    let filter = if cli.verbose {
        EnvFilter::new(Level::DEBUG.to_string())
    } else {
        EnvFilter::from_default_env().add_directive(Level::WARN.into())
    };

    tracing_subscriber::fmt().with_env_filter(filter).with_target(false).init();

    // Load config with layered overrides
    let mut config = Config::load(cli.config.as_deref())?.with_env();

    // Apply CLI overrides
    config.region = cli.region;
    config.format = cli.format;
    config.delay_ms = cli.delay;

    if let Some(proxy) = cli.proxy {
        config.proxy = Some(proxy);
    }

    match cli.command {
        Commands::Search {
            query,
            max,
            min_price,
            max_price,
            min_rating,
            prime_only,
            no_sponsored,
            keywords,
            exclude,
        } => {
            // Apply search-specific config
            config.max_results = max;
            config.min_price = min_price;
            config.max_price = max_price;
            config.min_rating = min_rating;
            config.prime_only = prime_only;
            config.no_sponsored = no_sponsored;

            if let Some(kw) = keywords {
                config.keywords = kw;
            }
            if let Some(ex) = exclude {
                config.exclude_keywords = ex;
            }

            let cmd = SearchCommand::new(config);
            let output = cmd.execute(&query).await?;
            println!("{}", output);
        }

        Commands::Product { asins } => {
            let cmd = ProductCommand::new(config);

            let output = if asins.len() == 1 {
                cmd.execute(&asins[0]).await?
            } else {
                cmd.execute_batch(&asins).await?
            };

            println!("{}", output);
        }

        Commands::Regions => {
            println!("Supported Amazon regions:\n");
            println!("{:<6} {:<20} {:<10}", "Code", "Domain", "Currency");
            println!("{:-<6} {:-<20} {:-<10}", "", "", "");

            for region in Region::all() {
                println!(
                    "{:<6} {:<20} {:<10}",
                    region.to_string(),
                    region.domain(),
                    region.currency()
                );
            }
        }

        #[cfg(feature = "tropical")]
        Commands::Compare { asin } => {
            use amz_crawler::commands::compare;
            let output = compare::compare_prices(&asin, config.format).await?;
            println!("{}", output);
        }

        #[cfg(feature = "tropical")]
        Commands::Tropical { query, max } => {
            use amz_crawler::commands::compare;
            let output = compare::search_tropical(&query, max, config.format).await?;
            println!("{}", output);
        }
    }

    Ok(())
}
