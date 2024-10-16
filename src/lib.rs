pub mod cli;

use serde::{Deserialize, Serialize};

pub const BASE_URL: &str = "https://brisbane.waste-info.com.au/api/v1/";

pub fn get_url(filename: &str) -> String {
    format!("{}{}", BASE_URL, filename)
}

#[repr(u8)]
#[derive(Serialize, Deserialize, Debug)]
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

// impl TryFrom<u8> for CollectionDays {
//     type Error = String;
//     fn try_from(item: u8) -> Result<Self, Self::Error> {
//         match item {
//             0 => Ok(CollectionDays::Sunday),
//             1 => Ok(CollectionDays::Monday),
//             2 => Ok(CollectionDays::Tuesday),
//             3 => Ok(CollectionDays::Wednesday),
//             4 => Ok(CollectionDays::Thursday),
//             5 => Ok(CollectionDays::Friday),
//             6 => Ok(CollectionDays::Saturday),
//             _ => Err("Invalid day of the week".to_string()),
//         }
//     }
// }

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

#[derive(Serialize, Deserialize, Debug)]
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

#[derive(Serialize, Deserialize, Debug)]
#[allow(dead_code)]
pub struct BinProperty {
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

#[derive(Serialize, Deserialize, Debug)]
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
