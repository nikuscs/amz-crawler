//! Configuration management with TOML, environment variables, and CLI overrides.

use crate::amazon::regions::Region;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;
use tracing::debug;

/// Application configuration with layered loading.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Amazon region
    #[serde(default)]
    pub region: Region,

    /// Proxy URL (e.g., socks5://host:port)
    #[serde(default)]
    pub proxy: Option<String>,

    /// Base delay between requests in milliseconds
    #[serde(default = "default_delay_ms")]
    pub delay_ms: u64,

    /// Random jitter added to delay (0 to this value)
    #[serde(default = "default_delay_jitter_ms")]
    pub delay_jitter_ms: u64,

    /// Maximum number of results to fetch
    #[serde(default = "default_max_results")]
    pub max_results: usize,

    /// Output format
    #[serde(default)]
    pub format: OutputFormat,

    /// Filter: minimum price
    #[serde(default)]
    pub min_price: Option<f64>,

    /// Filter: maximum price
    #[serde(default)]
    pub max_price: Option<f64>,

    /// Filter: minimum rating
    #[serde(default)]
    pub min_rating: Option<f32>,

    /// Filter: Prime-only products
    #[serde(default)]
    pub prime_only: bool,

    /// Filter: exclude sponsored products
    #[serde(default)]
    pub no_sponsored: bool,

    /// Filter: keywords that must appear in title
    #[serde(default)]
    pub keywords: Vec<String>,

    /// Filter: keywords that must NOT appear in title
    #[serde(default)]
    pub exclude_keywords: Vec<String>,
}

fn default_delay_ms() -> u64 {
    2000
}

fn default_delay_jitter_ms() -> u64 {
    3000
}

fn default_max_results() -> usize {
    20
}

impl Default for Config {
    fn default() -> Self {
        Self {
            region: Region::Us,
            proxy: None,
            delay_ms: default_delay_ms(),
            delay_jitter_ms: default_delay_jitter_ms(),
            max_results: default_max_results(),
            format: OutputFormat::Table,
            min_price: None,
            max_price: None,
            min_rating: None,
            prime_only: false,
            no_sponsored: false,
            keywords: Vec::new(),
            exclude_keywords: Vec::new(),
        }
    }
}

impl Config {
    /// Creates a new default configuration.
    pub fn new() -> Self {
        Self::default()
    }

    /// Loads configuration from a TOML file.
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        debug!("Loading config from: {}", path.display());

        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;

        toml::from_str(&content)
            .with_context(|| format!("Failed to parse config file: {}", path.display()))
    }

    /// Loads configuration with fallback to default locations.
    pub fn load(explicit_path: Option<&Path>) -> Result<Self> {
        // 1. Explicit path takes precedence
        if let Some(path) = explicit_path {
            return Self::from_file(path);
        }

        // 2. Try current directory
        let local_config = Path::new("config.toml");
        if local_config.exists() {
            debug!("Found config.toml in current directory");
            return Self::from_file(local_config);
        }

        // 3. Try XDG config directory
        if let Some(config_dir) = dirs::config_dir() {
            let xdg_config = config_dir.join("amz-crawler").join("config.toml");
            if xdg_config.exists() {
                debug!("Found config in XDG config directory");
                return Self::from_file(xdg_config);
            }
        }

        // 4. Return default config
        debug!("No config file found, using defaults");
        Ok(Self::default())
    }

    /// Applies environment variable overrides.
    pub fn with_env(mut self) -> Self {
        if let Ok(region) = std::env::var("AMZ_REGION") {
            if let Ok(r) = region.parse() {
                self.region = r;
            }
        }

        if let Ok(proxy) = std::env::var("AMZ_PROXY") {
            self.proxy = Some(proxy);
        }

        if let Ok(delay) = std::env::var("AMZ_DELAY") {
            if let Ok(d) = delay.parse() {
                self.delay_ms = d;
            }
        }

        self
    }
}

/// Output format for results.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    #[default]
    Table,
    Json,
    Markdown,
    Csv,
}

impl std::str::FromStr for OutputFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "table" => Ok(OutputFormat::Table),
            "json" => Ok(OutputFormat::Json),
            "markdown" | "md" => Ok(OutputFormat::Markdown),
            "csv" => Ok(OutputFormat::Csv),
            _ => Err(format!("Unknown format: {}. Use: table, json, markdown, csv", s)),
        }
    }
}

impl std::fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OutputFormat::Table => write!(f, "table"),
            OutputFormat::Json => write!(f, "json"),
            OutputFormat::Markdown => write!(f, "markdown"),
            OutputFormat::Csv => write!(f, "csv"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.region, Region::Us);
        assert_eq!(config.delay_ms, 2000);
        assert_eq!(config.delay_jitter_ms, 3000);
        assert_eq!(config.max_results, 20);
        assert_eq!(config.format, OutputFormat::Table);
        assert!(config.proxy.is_none());
        assert!(config.min_price.is_none());
        assert!(config.max_price.is_none());
        assert!(config.min_rating.is_none());
        assert!(!config.prime_only);
        assert!(!config.no_sponsored);
        assert!(config.keywords.is_empty());
        assert!(config.exclude_keywords.is_empty());
    }

    #[test]
    fn test_config_new() {
        let config = Config::new();
        assert_eq!(config.region, Region::Us);
        assert_eq!(config.delay_ms, 2000);
    }

    #[test]
    fn test_output_format_parsing() {
        assert_eq!("table".parse::<OutputFormat>().unwrap(), OutputFormat::Table);
        assert_eq!("TABLE".parse::<OutputFormat>().unwrap(), OutputFormat::Table);
        assert_eq!("json".parse::<OutputFormat>().unwrap(), OutputFormat::Json);
        assert_eq!("JSON".parse::<OutputFormat>().unwrap(), OutputFormat::Json);
        assert_eq!("markdown".parse::<OutputFormat>().unwrap(), OutputFormat::Markdown);
        assert_eq!("md".parse::<OutputFormat>().unwrap(), OutputFormat::Markdown);
        assert_eq!("MD".parse::<OutputFormat>().unwrap(), OutputFormat::Markdown);
        assert_eq!("csv".parse::<OutputFormat>().unwrap(), OutputFormat::Csv);
        assert_eq!("CSV".parse::<OutputFormat>().unwrap(), OutputFormat::Csv);

        let err = "invalid".parse::<OutputFormat>().unwrap_err();
        assert!(err.contains("Unknown format"));
        assert!(err.contains("table, json, markdown, csv"));
    }

    #[test]
    fn test_output_format_display() {
        assert_eq!(OutputFormat::Table.to_string(), "table");
        assert_eq!(OutputFormat::Json.to_string(), "json");
        assert_eq!(OutputFormat::Markdown.to_string(), "markdown");
        assert_eq!(OutputFormat::Csv.to_string(), "csv");
    }

    #[test]
    fn test_output_format_default() {
        assert_eq!(OutputFormat::default(), OutputFormat::Table);
    }

    #[test]
    fn test_output_format_serde() {
        let format = OutputFormat::Json;
        let json = serde_json::to_string(&format).unwrap();
        assert_eq!(json, "\"json\"");

        let parsed: OutputFormat = serde_json::from_str("\"markdown\"").unwrap();
        assert_eq!(parsed, OutputFormat::Markdown);
    }

    #[test]
    fn test_config_from_toml() {
        let toml = r#"
            region = "uk"
            delay_ms = 3000
            max_results = 50
            prime_only = true
        "#;

        let config: Config = toml::from_str(toml).unwrap();
        assert_eq!(config.region, Region::Uk);
        assert_eq!(config.delay_ms, 3000);
        assert_eq!(config.max_results, 50);
        assert!(config.prime_only);
    }

    #[test]
    fn test_config_from_toml_all_fields() {
        let toml = r#"
            region = "de"
            proxy = "socks5://localhost:1080"
            delay_ms = 5000
            delay_jitter_ms = 2000
            max_results = 100
            format = "json"
            min_price = 10.0
            max_price = 100.0
            min_rating = 4.0
            prime_only = true
            no_sponsored = true
            keywords = ["gaming", "rgb"]
            exclude_keywords = ["refurbished", "used"]
        "#;

        let config: Config = toml::from_str(toml).unwrap();
        assert_eq!(config.region, Region::De);
        assert_eq!(config.proxy, Some("socks5://localhost:1080".to_string()));
        assert_eq!(config.delay_ms, 5000);
        assert_eq!(config.delay_jitter_ms, 2000);
        assert_eq!(config.max_results, 100);
        assert_eq!(config.format, OutputFormat::Json);
        assert_eq!(config.min_price, Some(10.0));
        assert_eq!(config.max_price, Some(100.0));
        assert_eq!(config.min_rating, Some(4.0));
        assert!(config.prime_only);
        assert!(config.no_sponsored);
        assert_eq!(config.keywords, vec!["gaming", "rgb"]);
        assert_eq!(config.exclude_keywords, vec!["refurbished", "used"]);
    }

    #[test]
    fn test_config_from_file() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            r#"
            region = "fr"
            delay_ms = 4000
            "#
        )
        .unwrap();

        let config = Config::from_file(file.path()).unwrap();
        assert_eq!(config.region, Region::Fr);
        assert_eq!(config.delay_ms, 4000);
    }

    #[test]
    fn test_config_from_file_not_found() {
        let result = Config::from_file("/nonexistent/path/config.toml");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Failed to read config file"));
    }

    #[test]
    fn test_config_from_file_invalid_toml() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "not valid toml {{{{").unwrap();

        let result = Config::from_file(file.path());
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Failed to parse config file"));
    }

    #[test]
    fn test_config_load_no_file() {
        // When no file exists, should return default config
        let config = Config::load(None).unwrap();
        assert_eq!(config.region, Region::Us);
    }

    #[test]
    fn test_config_load_explicit_path() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            r#"
            region = "jp"
            max_results = 30
            "#
        )
        .unwrap();

        let config = Config::load(Some(file.path())).unwrap();
        assert_eq!(config.region, Region::Jp);
        assert_eq!(config.max_results, 30);
    }

    #[test]
    fn test_config_with_env() {
        // Save original env vars
        let orig_region = std::env::var("AMZ_REGION").ok();
        let orig_proxy = std::env::var("AMZ_PROXY").ok();
        let orig_delay = std::env::var("AMZ_DELAY").ok();

        // Set test env vars
        std::env::set_var("AMZ_REGION", "au");
        std::env::set_var("AMZ_PROXY", "http://proxy:8080");
        std::env::set_var("AMZ_DELAY", "5000");

        let config = Config::new().with_env();
        assert_eq!(config.region, Region::Au);
        assert_eq!(config.proxy, Some("http://proxy:8080".to_string()));
        assert_eq!(config.delay_ms, 5000);

        // Restore original env vars
        match orig_region {
            Some(v) => std::env::set_var("AMZ_REGION", v),
            None => std::env::remove_var("AMZ_REGION"),
        }
        match orig_proxy {
            Some(v) => std::env::set_var("AMZ_PROXY", v),
            None => std::env::remove_var("AMZ_PROXY"),
        }
        match orig_delay {
            Some(v) => std::env::set_var("AMZ_DELAY", v),
            None => std::env::remove_var("AMZ_DELAY"),
        }
    }

    #[test]
    fn test_config_with_env_invalid_values() {
        let orig_region = std::env::var("AMZ_REGION").ok();
        let orig_delay = std::env::var("AMZ_DELAY").ok();

        // Set invalid values
        std::env::set_var("AMZ_REGION", "invalid_region");
        std::env::set_var("AMZ_DELAY", "not_a_number");

        let config = Config::new().with_env();
        // Invalid values should be ignored, keeping defaults
        assert_eq!(config.region, Region::Us);
        assert_eq!(config.delay_ms, 2000);

        // Restore
        match orig_region {
            Some(v) => std::env::set_var("AMZ_REGION", v),
            None => std::env::remove_var("AMZ_REGION"),
        }
        match orig_delay {
            Some(v) => std::env::set_var("AMZ_DELAY", v),
            None => std::env::remove_var("AMZ_DELAY"),
        }
    }

    #[test]
    fn test_config_serde_roundtrip() {
        let config = Config {
            region: Region::Uk,
            proxy: Some("socks5://localhost:1080".to_string()),
            delay_ms: 3000,
            delay_jitter_ms: 1500,
            max_results: 50,
            format: OutputFormat::Json,
            min_price: Some(10.0),
            max_price: Some(100.0),
            min_rating: Some(4.0),
            prime_only: true,
            no_sponsored: true,
            keywords: vec!["test".to_string()],
            exclude_keywords: vec!["exclude".to_string()],
        };

        let json = serde_json::to_string(&config).unwrap();
        let parsed: Config = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.region, config.region);
        assert_eq!(parsed.proxy, config.proxy);
        assert_eq!(parsed.delay_ms, config.delay_ms);
        assert_eq!(parsed.max_results, config.max_results);
        assert_eq!(parsed.format, config.format);
        assert_eq!(parsed.min_price, config.min_price);
        assert_eq!(parsed.prime_only, config.prime_only);
    }
}
