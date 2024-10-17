use clap::Parser;

#[derive(Parser)]
#[command(version, about)]
/// Parses the Brisbane City Council bin data API
pub struct Cli {
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
}

pub struct AddressData {
    pub street: String,
    pub address: String,
    pub suburb: String,
}

impl Cli {
    pub fn get_data(&self) -> Result<AddressData, String> {
        let address = match &self.address {
            Some(s) => s,
            None => return Err("No address provided".to_string()),
        };
        let mut res = address.split(',').map(|s| s.trim());
        let address = match res.next() {
            Some(s) => s,
            None => {
                return Err(
                    "No street provided, specify address like 123 drury lane, suburb".to_string(),
                )
            }
        };

        let street = address
            .split_whitespace()
            .skip(1)
            .collect::<Vec<&str>>()
            .join(" ");

        let suburb = match res.next() {
            Some(s) => s,
            None => {
                return Err(
                    "No suburb provided, specify address like 123 drury lane, suburb".to_string(),
                )
            }
        };
        Ok(AddressData {
            street: street.to_string(),
            address: address.to_string(),
            suburb: suburb.to_string(),
        })
    }
}
