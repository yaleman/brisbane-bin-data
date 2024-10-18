pub mod cli;

use std::fmt::Display;

use serde::{Deserialize, Serialize};
use time::{format_description, Date};

pub const BASE_URL: &str = "https://brisbane.waste-info.com.au/api/v1/";

pub fn get_url(filename: &str) -> String {
    format!("{}{}", BASE_URL, filename)
}

#[repr(u8)]
#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Hash)]
#[serde(from = "u8")]
pub enum CollectionDay {
    Sunday,
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
}

impl From<u8> for CollectionDay {
    fn from(item: u8) -> Self {
        match item {
            0 => CollectionDay::Sunday,
            1 => CollectionDay::Monday,
            2 => CollectionDay::Tuesday,
            3 => CollectionDay::Wednesday,
            4 => CollectionDay::Thursday,
            5 => CollectionDay::Friday,
            6 => CollectionDay::Saturday,
            _ => panic!("Invalid day of the week"),
        }
    }
}

impl std::fmt::Display for CollectionDay {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let day = format!("{:?}", self);
        write!(f, "{}", day.split("::").last().ok_or(std::fmt::Error)?)
    }
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
pub struct Locality {
    pub id: u32,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub postcode: Option<String>,
    pub council: String,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
pub struct Localities {
    pub localities: Vec<Locality>,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
pub struct Street {
    pub id: u32,
    pub name: String,
    pub locality: String,
}

#[derive(Deserialize, Debug)]
pub struct Streets {
    pub streets: Vec<Street>,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
pub struct Property {
    pub id: u64,
    pub name: String,
    pub zone: String,
    pub voucher_preferences: u32,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
pub struct Properties {
    pub properties: Vec<Property>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[allow(dead_code)]
pub struct BinDay {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub color: String,
    #[serde(rename = "textColor")]
    pub text_colour: String,
    #[serde(rename = "borderColor")]
    pub border_colour: String,
    pub start: String,
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
            write!(f, "{} ", name)?
        }
        write!(f, "{}", self.event_type)
    }
}

impl BinDay {
    pub fn get_start_date(&self) -> Result<Date, String> {
        let formatter =
            format_description::parse("[year]-[month]-[day]").map_err(|err| err.to_string())?;
        Date::parse(&self.start, &formatter).map_err(|err| err.to_string())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[allow(dead_code)]
pub struct BinProperty {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<u64>,
    pub collection_day: CollectionDay,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub collection_day_2: Option<CollectionDay>,
    pub zone: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shs: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bin_bank_id: Option<String>,
    pub clean_up_code: String,
    pub address: String,
    pub service_type: String,
    pub collections: Vec<String>,
}

impl Display for BinProperty {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if let Some(id) = self.id {
            writeln!(f, "Property ID: {}", id)?;
        }
        writeln!(f, "Collection Day: {}", self.collection_day)?;
        if let Some(collection_day_2) = &self.collection_day_2 {
            writeln!(f, "Collection Day 2: {}", collection_day_2)?;
        }
        if let Some(bin_bank_id) = &self.bin_bank_id {
            writeln!(f, "Bin Bank ID: {}", bin_bank_id)?;
        }
        if let Some(shs) = &self.shs {
            writeln!(f, "SHS: {}", shs)?;
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
#[allow(dead_code)]
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
