//! Brisbane Bin Data
//! A simple CLI to query the Brisbane City Council bin data API
//!

#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![deny(warnings)]
#![deny(clippy::all)]
#![deny(clippy::await_holding_lock)]
#![deny(clippy::complexity)]
#![deny(clippy::correctness)]
#![deny(clippy::expect_used)]
#![deny(clippy::needless_pass_by_value)]
#![deny(clippy::panic)]
#![deny(clippy::trivially_copy_pass_by_ref)]
#![deny(clippy::unreachable)]
#![deny(clippy::unwrap_used)]

pub mod cli;

use std::{fmt::Display, str::FromStr};

use reqwest::Url;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use time::{format_description, util::days_in_month, Date};

/// The base URL for the Brisbane City Council bin data API
pub const BASE_URL: &str = "https://brisbane.waste-info.com.au/api/v1/";

/// Get the full URL for a given endpoint
pub fn get_url(filename: &str) -> String {
    format!("{BASE_URL}{filename}")
}

#[repr(u8)]
#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Hash)]
#[serde(try_from = "u8")]
#[allow(missing_docs)]
/// The days of the week for bin collection
pub enum CollectionDay {
    Sunday,
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
}

impl TryFrom<u8> for CollectionDay {
    type Error = String;
    fn try_from(item: u8) -> Result<Self, Self::Error> {
        match item {
            0 => Ok(CollectionDay::Sunday),
            1 => Ok(CollectionDay::Monday),
            2 => Ok(CollectionDay::Tuesday),
            3 => Ok(CollectionDay::Wednesday),
            4 => Ok(CollectionDay::Thursday),
            5 => Ok(CollectionDay::Friday),
            6 => Ok(CollectionDay::Saturday),
            _ => Err(format!("Invalid day of the week: {}", item)),
        }
    }
}

impl std::fmt::Display for CollectionDay {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let day = format!("{self:?}");
        write!(f, "{}", day.split("::").last().ok_or(std::fmt::Error)?)
    }
}

#[derive(Deserialize, Debug)]

/// The locality data
pub struct Locality {
    /// The locality ID, used for querying streets
    pub id: u32,
    /// The locality name, e.g. "Brisbane"
    pub name: String,
    /// The locality postcode, e.g. "4000", may be null
    #[serde(skip_serializing_if = "Option::is_none")]
    pub postcode: Option<String>,
    /// The council name, e.g. "Brisbane City Council"
    pub council: String,
}

#[derive(Deserialize, Debug)]
/// The list of localities, which can be queried for streets, used for deserializing the API response
pub struct Localities {
    /// The list of localities, which can be queried for streets
    pub localities: Vec<Locality>,
}

#[derive(Deserialize, Debug)]
/// The street data
pub struct Street {
    /// The street ID, used for querying properties
    pub id: u32,
    /// The street name, e.g. "Drury Lane"
    pub name: String,
    /// The locality name, e.g. "Brisbane"
    pub locality: String,
}

#[derive(Deserialize, Debug)]
/// The list of streets, which can be queried for properties, used for deserializing the API response
pub struct Streets {
    /// The list of streets, which can be queried for properties
    pub streets: Vec<Street>,
}

#[derive(Deserialize, Debug)]
/// The property data
pub struct Property {
    /// The property ID, used for querying bin data
    pub id: u64,
    /// The property name, e.g. "123 Drury Lane"
    pub name: String,
    /// The property zone, e.g. "Zone 1"
    pub zone: String,
    /// The property voucher preferences, e.g. 2
    pub voucher_preferences: u32,
}

#[derive(Deserialize, Debug)]
/// Desrializes the list of properties, which can be queried for bin data, used for deserializing the API response
pub struct Properties {
    /// The list of properties, which can be queried for bin data
    pub properties: Vec<Property>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
/// The bin day data
pub struct BinDay {
    /// The bin day ID, used for querying bin data, may be null
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<u32>,
    /// The bin day name, e.g. "Green Bin", may be null
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// The bin day description, e.g. "Green bin collection", may be null
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// The bin day color, e.g. "#00FF00"
    pub color: String,
    /// The bin day text color, e.g. "#000000"
    #[serde(rename = "textColor")]
    pub text_colour: String,
    /// The bin day border color, e.g. "#FFFFFF"
    #[serde(rename = "borderColor")]
    pub border_colour: String,
    /// The bin day start date, e.g. "2024-01-01"
    pub start: String,
    /// The bin day event type, e.g. "collection"
    pub event_type: String,
}

impl std::fmt::Display for BinDay {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{} ",
            self.get_start_date()
                .map(|val| val.to_string())
                .unwrap_or(self.start.clone())
        )?;
        if let Some(name) = &self.name {
            write!(f, "{name} ")?
        }
        write!(f, "{}", self.event_type)
    }
}

impl BinDay {
    /// Get the start date as a `time::Date`, parsing it from the `start` string field. The expected format is "YYYY-MM-DD".
    pub fn get_start_date(&self) -> Result<Date, String> {
        let formatter = format_description::parse_owned::<2>("[year]-[month]-[day]")
            .map_err(|err| err.to_string())?;
        Date::parse(&self.start, &formatter).map_err(|err| err.to_string())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
/// The bin property data
pub struct BinProperty {
    /// The property ID, used for querying bin data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<u64>,
    /// The primary collection day
    pub collection_day: CollectionDay,
    /// The secondary collection day, may be null
    #[serde(skip_serializing_if = "Option::is_none")]
    pub collection_day_2: Option<CollectionDay>,
    /// The property zone, e.g. "Zone 1"
    pub zone: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// The SHS code, e.g. "SHS123"?
    pub shs: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// The bin bank ID, e.g. "BB123"?
    pub bin_bank_id: Option<String>,
    /// The clean up code, e.g. "C123"
    pub clean_up_code: String,
    /// The property address, e.g. "123 Drury Lane"
    pub address: String,
    /// The service type, e.g. "Garbage"
    pub service_type: String,
    /// The collection days, e.g. ["Monday", "Thursday"]
    pub collections: Vec<String>,
}

impl Display for BinProperty {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if let Some(id) = self.id {
            writeln!(f, "Property ID: {id}")?;
        }
        writeln!(f, "Collection Day: {}", self.collection_day)?;
        if let Some(collection_day_2) = &self.collection_day_2 {
            writeln!(f, "Collection Day 2: {collection_day_2}")?;
        }
        if let Some(bin_bank_id) = &self.bin_bank_id {
            writeln!(f, "Bin Bank ID: {bin_bank_id}")?;
        }
        if let Some(shs) = &self.shs {
            writeln!(f, "SHS: {shs}")?;
        }
        write!(f, "Service Type: {}", self.service_type)?;
        if !self.collections.is_empty() {
            write!(f, "({})", self.collections.join(", "))?;
        }
        writeln!(f, " ")?;
        writeln!(f, "Address: {} ({})", self.address, self.zone)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
/// The complete bin data for a property, including the property data and the associated bin days
#[allow(missing_docs)]
pub struct BinData {
    pub property: BinProperty,
    pub color: String,
    #[serde(rename = "textColor")]
    pub text_colour: String,
    #[serde(rename = "borderColor")]
    pub border_colour: String,

    pub dow: Vec<CollectionDay>,
    #[serde(rename = "daysOfWeek")]
    pub days_of_week: Vec<CollectionDay>,
    pub start_date: String,
    pub event_type: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
/// The error response from the API, used for deserializing error responses
pub struct ErrorResponse {
    /// The error message
    pub error: String,
    /// The HTTP status code
    pub status: u16,
}

/// The parsed address data
#[derive(Debug)]
pub struct AddressData {
    /// The street number, e.g. "123"
    pub num: String,
    /// The street name, without the number, e.g. "drury lane"
    pub street: String,
    /// The full address, including the number, e.g. "123 drury lane"
    pub address: String,
    /// The suburb, e.g. "brisbane"
    pub suburb: String,
}

impl TryFrom<String> for AddressData {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let mut res = value.split(',').map(|s| s.trim());
        let Some(address) = res.next() else {
            return Err(
                "No street provided, specify address like 123 drury lane, suburb".to_string(),
            );
        };
        let mut splitup = address.split_whitespace();
        let Some(num) = splitup.next() else {
            return Err(
                "No street number provided, specify address like 123 drury lane, suburb"
                    .to_string(),
            );
        };

        let street = splitup.collect::<Vec<&str>>().join(" ");

        let suburb = match res.next() {
            Some(s) => s,
            None => {
                return Err(
                    "No suburb provided, specify address like 123 drury lane, suburb".to_string(),
                )
            }
        };
        Ok(AddressData {
            num: num.to_string(),
            street: street.to_string(),
            address: address.to_string(),
            suburb: suburb.to_string(),
        })
    }
}

/// The main data structure for interacting with the API
pub struct BinClient {
    client: reqwest::Client,
    debug: bool,
}

impl Default for BinClient {
    fn default() -> Self {
        Self {
            client: reqwest::Client::new(),
            debug: false,
        }
    }
}

impl BinClient {
    /// Builder method
    pub fn with_debug(self) -> Self {
        Self {
            debug: true,
            ..self
        }
    }
    /// Get the list of localities, which can be queried for streets
    pub async fn get_localities(&mut self) -> Result<Vec<Locality>, String> {
        let url = get_url("localities.json");
        // -H 'Authorization: Token token="<32 hex chars>"' \
        // -H 'Origin: https://impact-apps-calendars.web.app' \
        // -H 'Referer: https://impact-apps-calendars.web.app/' \
        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|err| format!("Failed to query localities {err:?}"))?;

        let localities: Localities = response
            .json()
            .await
            .map_err(|err| format!("Failed to parse localities {err:?}"))?;

        Ok(localities.localities)
    }

    /// Get the list of streets for a given locality, which can be queried for properties
    pub async fn get_streets(&mut self, locality_id: &u32) -> Result<Vec<Street>, String> {
        let mut url = Url::from_str(&get_url("streets.json"))
            .map_err(|err| format!("Failed to make streets URL {err:?}"))?;
        url.query_pairs_mut()
            .append_pair("locality", &locality_id.to_string());
        let response = self
            .client
            .get(url.as_str())
            .send()
            .await
            .map_err(|err| format!("Failed to query streets {err:?}"))?;

        let streets: Streets = response
            .json()
            .await
            .map_err(|err| format!("Failed to parse streets JSON {err:?}"))?;

        Ok(streets.streets)
    }

    /// Get the list of properties for a given street, which can be queried for bin data
    pub async fn get_properties(&mut self, street_id: &u32) -> Result<Vec<Property>, String> {
        let mut url = reqwest::Url::from_str(&get_url("properties.json"))
            .map_err(|err| format!("Failed to make properties URL {err:?}"))?;
        url.query_pairs_mut()
            .append_pair("street", &street_id.to_string());
        let response = self
            .client
            .get(url.as_str())
            .send()
            .await
            .map_err(|err| format!("Failed to query properties {err:?}"))?;

        let properties: Properties = response
            .json()
            .await
            .map_err(|err| format!("Failed to parse properties JSON {err:?}"))?;

        Ok(properties.properties)
    }

    /// Get the bin data for a given property ID, including the property data and the associated bin days
    pub async fn get_property(
        &mut self,
        property_id: u64,
    ) -> Result<(BinData, Vec<BinDay>), String> {
        let mut url = Url::from_str(&get_url(&format!("properties/{property_id}.json")))
            .map_err(|err| format!("Failed to create property URL! {err:?}"))?;

        let mut url_query = url.query_pairs_mut();

        let date_formatter = format_description::parse_borrowed::<2>(
            "[year]-[month]-[day]T[hour]:[minute]:[second]Z",
        )
        .map_err(|err| format!("Failed to generate date formatter! {err:?}"))?;

        // get the last day of the previous month
        let start = time::OffsetDateTime::now_utc()
            - time::Duration::days(time::OffsetDateTime::now_utc().day() as i64);
        let start = start.replace_time(
            time::Time::from_hms(14, 0, 0)
                .map_err(|err| format!("Failed to generate end date! {err:?}"))?,
        );

        let start = start
            .format(&date_formatter)
            .map_err(|err| format!("failed to string-format start date! {err:?}"))?;
        if self.debug {
            eprintln!("Start date: {start}");
        }

        let end = time::OffsetDateTime::now_utc();
        let days_of_month = days_in_month(end.month(), end.year());
        let end = end.replace_date(
            time::Date::from_calendar_date(end.year(), end.month(), days_of_month)
                .map_err(|err| format!("Failed to calculate end date! {err:?}"))?,
        );
        let end = end
            .format(&date_formatter)
            .map_err(|err| format!("Failed to string-format end date! {err:?}"))?;
        if self.debug {
            eprintln!("End date: {end}");
        }

        url_query.append_pair("start", &start);
        url_query.append_pair("end", &end);
        drop(url_query);
        if self.debug {
            eprintln!("get_property URL: {url}");
        }

        let response = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|err| format!("Failed to query bin data for property {err:?}"))?;

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            if self.debug {
                eprintln!(
                    "Property with ID {property_id} not found: {:?}",
                    response.json::<Value>().await
                );
            }
            return Err(format!("Property with ID {property_id} not found"));
        }

        let data: Value = response
            .json()
            .await
            .map_err(|err| format!("Failed to parse bin data for property from JSON {err:?}"))?;

        if let Ok(error_response) = serde_json::from_value::<ErrorResponse>(data.clone()) {
            return Err(format!(
                "API returned an error for property {property_id}: {} (status {})",
                error_response.error, error_response.status
            ));
        }

        if let Some(data) = data.as_array() {
            let mut data = data.iter();
            let bin_property: Value = match data.next() {
                Some(val) => val.clone(),
                None => {
                    return Err(
                        "Failed to find the first result, which should be a property!".to_string(),
                    )
                }
            };

            let mut bin_property: BinData = serde_json::from_value(bin_property)
                .map_err(|err| format!("Failed to parse property data! {err:?}"))?;

            bin_property.property.id = Some(property_id);

            let mut bin_days: Vec<BinDay> = Vec::new();

            for day in data {
                bin_days.push(
                    serde_json::from_value(day.clone())
                        .map_err(|err| format!("Failed to parse a bin day! {err:?}"))?,
                );
            }

            Ok((bin_property, bin_days))
        } else {
            Err(format!(
                "Failed to parse bin data for property from JSON: expected an array: {data:?}"
            ))
        }
    }

    /// Get the bin data for a given address, including the property data and the associated bin days.
    pub async fn get_address(
        &mut self,
        address: AddressData,
    ) -> Result<(BinData, Vec<BinDay>), String> {
        let localities = match self.get_localities().await {
            Ok(val) => val,
            Err(e) => {
                eprintln!("Failed to query localities: {e}");
                return Err(format!("Failed to query localities: {e}"));
            }
        };

        let my_locality = match localities
            .iter()
            .find(|l| l.name.to_lowercase() == address.suburb.to_lowercase())
        {
            Some(l) => l,
            None => {
                return Err(format!("Suburb {} not found", address.suburb));
            }
        };
        if self.debug {
            println!("{my_locality:?}");
        }

        let streets = self
            .get_streets(&my_locality.id)
            .await
            .map_err(|e| format!("Failed to query streets: {e}"))?;

        let my_street: Street = match streets
            .into_iter()
            .find(|s| s.name.to_lowercase() == address.street.to_lowercase())
        {
            Some(s) => s,
            None => {
                return Err(format!("Street '{}' not found", address.street));
            }
        };
        let properties = self
            .get_properties(&my_street.id)
            .await
            .map_err(|e| format!("Failed to query properties: {e}"))?;

        let my_property = match properties.iter().find(|p| {
            p.name
                .to_lowercase()
                .contains(&address.address.to_lowercase())
        }) {
            Some(p) => p,
            None => {
                return Err(format!("Property {} not found", address.address));
            }
        };
        if self.debug {
            println!("{my_property:?}");
        }

        self.get_property(my_property.id)
            .await
            .map_err(|e| format!("Failed to get property: {e}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_invalid_property() {
        let mut client = super::BinClient::default().with_debug();
        let result = client.get_property(1).await;
        assert!(result.is_err());
    }
    #[tokio::test]
    async fn test_ok_property() {
        let mut client = super::BinClient::default().with_debug();
        let address = AddressData::try_from("2 Boundary St, West End".to_string())
            .expect("Failed to parse address");

        assert_eq!(address.street, "Boundary St");
        assert_eq!(address.suburb, "West End");

        let result = client.get_property(2695626).await;
        assert!(result.is_ok());

        let result = client
            .get_address(address)
            .await
            .expect("Failed to get address");
        assert_eq!(result.0.property.id, Some(2695626));
    }

    #[test]
    fn test_days() {
        for day in 0..=7 {
            let collection_day = CollectionDay::try_from(day);
            if day < 7 {
                assert!(collection_day.is_ok());
                let collection_day = collection_day.expect("But we just checked it should be ok!");
                let _ = format!("{}", collection_day);
                assert_eq!(collection_day as u8, day);
            } else {
                assert!(collection_day.is_err());
            }
        }
    }
}
