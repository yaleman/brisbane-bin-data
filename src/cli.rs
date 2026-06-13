//! Cli Interface
use clap::Parser;

use crate::AddressData;

#[derive(Parser)]
#[command(version, about)]
/// Parses the Brisbane City Council bin data API
pub struct Cli {
    /// The address to query, in the format "123 drury lane, suburb"
    pub address: Option<String>,
    #[clap(long)]
    /// Enable debug logging
    pub debug: bool,
    /// Just go straight to the data if you know your property ID (it's in the 'full' output)
    #[clap(long, short)]
    pub property_id: Option<u64>,

    /// Just show my bin day(s), might return a list if you're lucky enough to have more than one!
    #[clap(long, short)]
    pub show_day: bool,
    /// Just show future dates
    #[clap(long, short)]
    pub future: bool,
    /// Show pretty data instead of JSON
    #[clap(long, short = 'P')]
    pub pretty: bool,
}

impl Cli {
    /// Get the parsed address data
    pub fn get_data(&self) -> Result<AddressData, String> {
        if let Some(address) = &self.address {
            AddressData::try_from(address.clone())
        } else {
            Err("No address provided".to_string())
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_cli() {
        let testval =
            Cli::try_parse_from(["test", "123 drury lane, suburb"]).expect("Failed to parse CLI");
        let data = testval.get_data();
        assert!(data.is_ok(), "Failed to get data from CLI: {:?}", data);

        let address = data.expect("Failed to get address data");
        assert_eq!(address.address, "123 drury lane");
        assert_eq!(address.street, "drury lane");
        assert_eq!(address.suburb, "suburb");
        assert_eq!(address.num, "123");

        let testval = Cli::try_parse_from(["test"]).expect("Failed to parse CLI");
        assert!(testval.get_data().is_err());
    }
}
