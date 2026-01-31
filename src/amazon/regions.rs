//! Amazon regional domains and currency configuration.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Supported Amazon regions with their domains and currencies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Region {
    #[default]
    Us,
    Uk,
    De,
    Fr,
    Es,
    It,
    Ca,
    Au,
    Jp,
    In,
    Br,
    Mx,
    Nl,
    Se,
    Pl,
}

impl Region {
    /// Returns the Amazon domain for this region.
    pub fn domain(&self) -> &'static str {
        match self {
            Region::Us => "amazon.com",
            Region::Uk => "amazon.co.uk",
            Region::De => "amazon.de",
            Region::Fr => "amazon.fr",
            Region::Es => "amazon.es",
            Region::It => "amazon.it",
            Region::Ca => "amazon.ca",
            Region::Au => "amazon.com.au",
            Region::Jp => "amazon.co.jp",
            Region::In => "amazon.in",
            Region::Br => "amazon.com.br",
            Region::Mx => "amazon.com.mx",
            Region::Nl => "amazon.nl",
            Region::Se => "amazon.se",
            Region::Pl => "amazon.pl",
        }
    }

    /// Returns the base URL for this region.
    pub fn base_url(&self) -> String {
        format!("https://www.{}", self.domain())
    }

    /// Returns the currency code for this region.
    pub fn currency(&self) -> &'static str {
        match self {
            Region::Us => "USD",
            Region::Uk => "GBP",
            Region::De | Region::Fr | Region::Es | Region::It | Region::Nl => "EUR",
            Region::Ca => "CAD",
            Region::Au => "AUD",
            Region::Jp => "JPY",
            Region::In => "INR",
            Region::Br => "BRL",
            Region::Mx => "MXN",
            Region::Se => "SEK",
            Region::Pl => "PLN",
        }
    }

    /// Returns the Accept-Language header value for this region.
    pub fn accept_language(&self) -> &'static str {
        match self {
            Region::Us | Region::Ca | Region::Au => "en-US,en;q=0.9",
            Region::Uk => "en-GB,en;q=0.9",
            Region::De => "de-DE,de;q=0.9,en;q=0.8",
            Region::Fr => "fr-FR,fr;q=0.9,en;q=0.8",
            Region::Es | Region::Mx => "es-ES,es;q=0.9,en;q=0.8",
            Region::It => "it-IT,it;q=0.9,en;q=0.8",
            Region::Jp => "ja-JP,ja;q=0.9,en;q=0.8",
            Region::In => "en-IN,en;q=0.9,hi;q=0.8",
            Region::Br => "pt-BR,pt;q=0.9,en;q=0.8",
            Region::Nl => "nl-NL,nl;q=0.9,en;q=0.8",
            Region::Se => "sv-SE,sv;q=0.9,en;q=0.8",
            Region::Pl => "pl-PL,pl;q=0.9,en;q=0.8",
        }
    }

    /// Returns whether this region uses comma as decimal separator.
    pub fn uses_comma_decimal(&self) -> bool {
        matches!(
            self,
            Region::De
                | Region::Fr
                | Region::Es
                | Region::It
                | Region::Nl
                | Region::Se
                | Region::Pl
                | Region::Br
        )
    }

    /// Returns all supported regions.
    pub fn all() -> &'static [Region] {
        &[
            Region::Us,
            Region::Uk,
            Region::De,
            Region::Fr,
            Region::Es,
            Region::It,
            Region::Ca,
            Region::Au,
            Region::Jp,
            Region::In,
            Region::Br,
            Region::Mx,
            Region::Nl,
            Region::Se,
            Region::Pl,
        ]
    }
}

impl fmt::Display for Region {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let code = match self {
            Region::Us => "us",
            Region::Uk => "uk",
            Region::De => "de",
            Region::Fr => "fr",
            Region::Es => "es",
            Region::It => "it",
            Region::Ca => "ca",
            Region::Au => "au",
            Region::Jp => "jp",
            Region::In => "in",
            Region::Br => "br",
            Region::Mx => "mx",
            Region::Nl => "nl",
            Region::Se => "se",
            Region::Pl => "pl",
        };
        write!(f, "{}", code)
    }
}

impl FromStr for Region {
    type Err = RegionParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "us" | "usa" | "united states" => Ok(Region::Us),
            "uk" | "gb" | "united kingdom" => Ok(Region::Uk),
            "de" | "germany" => Ok(Region::De),
            "fr" | "france" => Ok(Region::Fr),
            "es" | "spain" => Ok(Region::Es),
            "it" | "italy" => Ok(Region::It),
            "ca" | "canada" => Ok(Region::Ca),
            "au" | "australia" => Ok(Region::Au),
            "jp" | "japan" => Ok(Region::Jp),
            "in" | "india" => Ok(Region::In),
            "br" | "brazil" => Ok(Region::Br),
            "mx" | "mexico" => Ok(Region::Mx),
            "nl" | "netherlands" => Ok(Region::Nl),
            "se" | "sweden" => Ok(Region::Se),
            "pl" | "poland" => Ok(Region::Pl),
            _ => Err(RegionParseError(s.to_string())),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RegionParseError(String);

impl fmt::Display for RegionParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Unknown region '{}'. Valid regions: us, uk, de, fr, es, it, ca, au, jp, in, br, mx, nl, se, pl",
            self.0
        )
    }
}

impl std::error::Error for RegionParseError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_region_parsing_all() {
        // Test all region codes
        assert_eq!(Region::from_str("us").unwrap(), Region::Us);
        assert_eq!(Region::from_str("usa").unwrap(), Region::Us);
        assert_eq!(Region::from_str("united states").unwrap(), Region::Us);
        assert_eq!(Region::from_str("uk").unwrap(), Region::Uk);
        assert_eq!(Region::from_str("gb").unwrap(), Region::Uk);
        assert_eq!(Region::from_str("united kingdom").unwrap(), Region::Uk);
        assert_eq!(Region::from_str("de").unwrap(), Region::De);
        assert_eq!(Region::from_str("germany").unwrap(), Region::De);
        assert_eq!(Region::from_str("fr").unwrap(), Region::Fr);
        assert_eq!(Region::from_str("france").unwrap(), Region::Fr);
        assert_eq!(Region::from_str("es").unwrap(), Region::Es);
        assert_eq!(Region::from_str("spain").unwrap(), Region::Es);
        assert_eq!(Region::from_str("it").unwrap(), Region::It);
        assert_eq!(Region::from_str("italy").unwrap(), Region::It);
        assert_eq!(Region::from_str("ca").unwrap(), Region::Ca);
        assert_eq!(Region::from_str("canada").unwrap(), Region::Ca);
        assert_eq!(Region::from_str("au").unwrap(), Region::Au);
        assert_eq!(Region::from_str("australia").unwrap(), Region::Au);
        assert_eq!(Region::from_str("jp").unwrap(), Region::Jp);
        assert_eq!(Region::from_str("japan").unwrap(), Region::Jp);
        assert_eq!(Region::from_str("in").unwrap(), Region::In);
        assert_eq!(Region::from_str("india").unwrap(), Region::In);
        assert_eq!(Region::from_str("br").unwrap(), Region::Br);
        assert_eq!(Region::from_str("brazil").unwrap(), Region::Br);
        assert_eq!(Region::from_str("mx").unwrap(), Region::Mx);
        assert_eq!(Region::from_str("mexico").unwrap(), Region::Mx);
        assert_eq!(Region::from_str("nl").unwrap(), Region::Nl);
        assert_eq!(Region::from_str("netherlands").unwrap(), Region::Nl);
        assert_eq!(Region::from_str("se").unwrap(), Region::Se);
        assert_eq!(Region::from_str("sweden").unwrap(), Region::Se);
        assert_eq!(Region::from_str("pl").unwrap(), Region::Pl);
        assert_eq!(Region::from_str("poland").unwrap(), Region::Pl);

        // Case insensitive
        assert_eq!(Region::from_str("US").unwrap(), Region::Us);
        assert_eq!(Region::from_str("GERMANY").unwrap(), Region::De);

        // Invalid
        assert!(Region::from_str("invalid").is_err());
        assert!(Region::from_str("").is_err());
    }

    #[test]
    fn test_region_domains_all() {
        assert_eq!(Region::Us.domain(), "amazon.com");
        assert_eq!(Region::Uk.domain(), "amazon.co.uk");
        assert_eq!(Region::De.domain(), "amazon.de");
        assert_eq!(Region::Fr.domain(), "amazon.fr");
        assert_eq!(Region::Es.domain(), "amazon.es");
        assert_eq!(Region::It.domain(), "amazon.it");
        assert_eq!(Region::Ca.domain(), "amazon.ca");
        assert_eq!(Region::Au.domain(), "amazon.com.au");
        assert_eq!(Region::Jp.domain(), "amazon.co.jp");
        assert_eq!(Region::In.domain(), "amazon.in");
        assert_eq!(Region::Br.domain(), "amazon.com.br");
        assert_eq!(Region::Mx.domain(), "amazon.com.mx");
        assert_eq!(Region::Nl.domain(), "amazon.nl");
        assert_eq!(Region::Se.domain(), "amazon.se");
        assert_eq!(Region::Pl.domain(), "amazon.pl");
    }

    #[test]
    fn test_region_base_url() {
        assert_eq!(Region::Us.base_url(), "https://www.amazon.com");
        assert_eq!(Region::Uk.base_url(), "https://www.amazon.co.uk");
        assert_eq!(Region::De.base_url(), "https://www.amazon.de");
    }

    #[test]
    fn test_region_currencies_all() {
        assert_eq!(Region::Us.currency(), "USD");
        assert_eq!(Region::Uk.currency(), "GBP");
        assert_eq!(Region::De.currency(), "EUR");
        assert_eq!(Region::Fr.currency(), "EUR");
        assert_eq!(Region::Es.currency(), "EUR");
        assert_eq!(Region::It.currency(), "EUR");
        assert_eq!(Region::Nl.currency(), "EUR");
        assert_eq!(Region::Ca.currency(), "CAD");
        assert_eq!(Region::Au.currency(), "AUD");
        assert_eq!(Region::Jp.currency(), "JPY");
        assert_eq!(Region::In.currency(), "INR");
        assert_eq!(Region::Br.currency(), "BRL");
        assert_eq!(Region::Mx.currency(), "MXN");
        assert_eq!(Region::Se.currency(), "SEK");
        assert_eq!(Region::Pl.currency(), "PLN");
    }

    #[test]
    fn test_accept_language_all() {
        assert!(Region::Us.accept_language().contains("en-US"));
        assert!(Region::Uk.accept_language().contains("en-GB"));
        assert!(Region::De.accept_language().contains("de-DE"));
        assert!(Region::Fr.accept_language().contains("fr-FR"));
        assert!(Region::Es.accept_language().contains("es-ES"));
        assert!(Region::It.accept_language().contains("it-IT"));
        assert!(Region::Ca.accept_language().contains("en-US"));
        assert!(Region::Au.accept_language().contains("en-US"));
        assert!(Region::Jp.accept_language().contains("ja-JP"));
        assert!(Region::In.accept_language().contains("en-IN"));
        assert!(Region::Br.accept_language().contains("pt-BR"));
        assert!(Region::Mx.accept_language().contains("es-ES"));
        assert!(Region::Nl.accept_language().contains("nl-NL"));
        assert!(Region::Se.accept_language().contains("sv-SE"));
        assert!(Region::Pl.accept_language().contains("pl-PL"));
    }

    #[test]
    fn test_comma_decimal_all() {
        // US-style (period decimal)
        assert!(!Region::Us.uses_comma_decimal());
        assert!(!Region::Uk.uses_comma_decimal());
        assert!(!Region::Ca.uses_comma_decimal());
        assert!(!Region::Au.uses_comma_decimal());
        assert!(!Region::Jp.uses_comma_decimal());
        assert!(!Region::In.uses_comma_decimal());
        assert!(!Region::Mx.uses_comma_decimal());

        // EU-style (comma decimal)
        assert!(Region::De.uses_comma_decimal());
        assert!(Region::Fr.uses_comma_decimal());
        assert!(Region::Es.uses_comma_decimal());
        assert!(Region::It.uses_comma_decimal());
        assert!(Region::Nl.uses_comma_decimal());
        assert!(Region::Se.uses_comma_decimal());
        assert!(Region::Pl.uses_comma_decimal());
        assert!(Region::Br.uses_comma_decimal());
    }

    #[test]
    fn test_region_all() {
        let all = Region::all();
        assert_eq!(all.len(), 15);
        assert!(all.contains(&Region::Us));
        assert!(all.contains(&Region::Pl));
    }

    #[test]
    fn test_region_display() {
        assert_eq!(Region::Us.to_string(), "us");
        assert_eq!(Region::Uk.to_string(), "uk");
        assert_eq!(Region::De.to_string(), "de");
        assert_eq!(Region::Fr.to_string(), "fr");
        assert_eq!(Region::Es.to_string(), "es");
        assert_eq!(Region::It.to_string(), "it");
        assert_eq!(Region::Ca.to_string(), "ca");
        assert_eq!(Region::Au.to_string(), "au");
        assert_eq!(Region::Jp.to_string(), "jp");
        assert_eq!(Region::In.to_string(), "in");
        assert_eq!(Region::Br.to_string(), "br");
        assert_eq!(Region::Mx.to_string(), "mx");
        assert_eq!(Region::Nl.to_string(), "nl");
        assert_eq!(Region::Se.to_string(), "se");
        assert_eq!(Region::Pl.to_string(), "pl");
    }

    #[test]
    fn test_region_default() {
        assert_eq!(Region::default(), Region::Us);
    }

    #[test]
    fn test_region_parse_error_display() {
        let err = Region::from_str("xyz").unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("xyz"));
        assert!(msg.contains("Valid regions"));
    }

    #[test]
    fn test_region_serde() {
        let region = Region::Us;
        let json = serde_json::to_string(&region).unwrap();
        assert_eq!(json, "\"us\"");

        let parsed: Region = serde_json::from_str("\"uk\"").unwrap();
        assert_eq!(parsed, Region::Uk);
    }
}
