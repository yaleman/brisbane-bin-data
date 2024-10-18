#![allow(dead_code)]

use std::collections::HashSet;
use std::str::FromStr;

use brisbane_bin_data::cli::{AddressData, Cli};
use clap::Parser;
use reqwest::Url;
use serde_json::Value;

use brisbane_bin_data::{
    get_url, BinData, BinDay, Localities, Locality, Properties, Property, Street, Streets,
};
use time::format_description;
use time::util::days_in_year_month;

struct Data {
    client: reqwest::Client,
}
impl Data {
    async fn get_localities(&mut self) -> Result<Vec<Locality>, String> {
        let url = get_url("localities.json");
        // -H 'Authorization: Token token="<32 hex chars>"' \
        // -H 'Origin: https://impact-apps-calendars.web.app' \
        // -H 'Referer: https://impact-apps-calendars.web.app/' \
        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|err| format!("Failed to query localities {:?}", err))?;

        let localities: Localities = response
            .json()
            .await
            .map_err(|err| format!("Failed to parse localities {:?}", err))?;

        Ok(localities.localities)
    }

    async fn get_streets(&mut self, locality_id: &u32) -> Result<Vec<Street>, String> {
        let mut url = Url::from_str(&get_url("streets.json"))
            .map_err(|err| format!("Failed to make streets URL {:?}", err))?;
        url.query_pairs_mut()
            .append_pair("locality", &locality_id.to_string());
        let response = self
            .client
            .get(url.as_str())
            .send()
            .await
            .map_err(|err| format!("Failed to query streets {:?}", err))?;

        let streets: Streets = response
            .json()
            .await
            .map_err(|err| format!("Failed to parse streets JSON {:?}", err))?;

        Ok(streets.streets)
    }
    async fn get_properties(&mut self, street_id: &u32) -> Result<Vec<Property>, String> {
        let mut url = reqwest::Url::from_str(&get_url("properties.json"))
            .map_err(|err| format!("Failed to make properties URL {:?}", err))?;
        url.query_pairs_mut()
            .append_pair("street", &street_id.to_string());
        let response = self
            .client
            .get(url.as_str())
            .send()
            .await
            .map_err(|err| format!("Failed to query properties {:?}", err))?;

        let properties: Properties = response
            .json()
            .await
            .map_err(|err| format!("Failed to parse properties JSON {:?}", err))?;

        Ok(properties.properties)
    }

    async fn get_property(
        &mut self,
        property_id: &u64,
        debug: bool,
    ) -> Result<(BinData, Vec<BinDay>), String> {
        let mut url = Url::from_str(&get_url(&format!("properties/{}.json", property_id)))
            .map_err(|err| format!("Failed to create property URL! {:?}", err))?;

        let mut url_query = url.query_pairs_mut();

        let date_formatter =
            format_description::parse("[year]-[month]-[day]T[hour]:[minute]:[second]Z")
                .map_err(|err| format!("Failed to generate date formatter! {:?}", err))?;

        // get the last day of the previous month
        let start = time::OffsetDateTime::now_utc()
            - time::Duration::days(time::OffsetDateTime::now_utc().day() as i64);
        let start = start.replace_time(
            time::Time::from_hms(14, 0, 0)
                .map_err(|err| format!("Failed to generate end date! {:?}", err))?,
        );

        let start = start
            .format(&date_formatter)
            .map_err(|err| format!("failed to string-format start date! {:?}", err))?;
        if debug {
            eprintln!("Start date: {}", start);
        }

        let end = time::OffsetDateTime::now_utc();
        let days_of_month = days_in_year_month(end.year(), end.month());
        let end = end.replace_date(
            time::Date::from_calendar_date(end.year(), end.month(), days_of_month)
                .map_err(|err| format!("Failed to calculate end date! {:?}", err))?,
        );
        let end = end
            .format(&date_formatter)
            .map_err(|err| format!("Failed to string-format end date! {:?}", err))?;
        if debug {
            eprintln!("End date: {}", end);
        }

        url_query.append_pair("start", &start);
        url_query.append_pair("end", &end);
        drop(url_query);
        if debug {
            eprintln!("get_property URL: {}", url);
        }

        let response = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|err| format!("Failed to query bin data for property {:?}", err))?;

        let data: Vec<Value> = response
            .json()
            .await
            .map_err(|err| format!("Failed to parse bin data for property from json {:?}", err))?;

        let mut data = data.into_iter();
        let bin_property: Value = match data.next() {
            Some(val) => val,
            None => {
                return Err(
                    "Failed to find the first result, which should be a property!".to_string(),
                )
            }
        };

        let mut bin_property: BinData = serde_json::from_value(bin_property)
            .map_err(|err| format!("Failed to parse property data! {:?}", err))?;

        bin_property.property.id = Some(*property_id);

        let mut bin_days: Vec<BinDay> = Vec::new();

        for day in data {
            bin_days.push(
                serde_json::from_value(day)
                    .map_err(|err| format!("Failed to parse a bin day! {:?}", err))?,
            );
        }

        Ok((bin_property, bin_days))
    }
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let mut client = Data {
        client: reqwest::Client::new(),
    };

    let bin_data = if let Some(property_id) = cli.property_id {
        match client.get_property(&property_id, cli.debug).await {
            Ok(val) => val,
            Err(e) => {
                eprintln!("{}", e);
                return;
            }
        }
    } else {
        if cli.address.is_none() {
            eprintln!("No address provided");
            return;
        }

        let address: AddressData = match cli.get_data() {
            Ok(val) => val,
            Err(e) => {
                eprintln!("{}", e);
                return;
            }
        };

        if cli.debug {
            eprintln!(
                "Searching for:\n Address: {}\n Street: {}\n Suburb: {}",
                address.address, address.street, address.suburb
            );
            eprintln!("Getting localities...");
        }
        let localities = match client.get_localities().await {
            Ok(val) => val,
            Err(e) => {
                eprintln!("Failed to query localities: {}", e);
                return;
            }
        };

        let my_locality = match localities.iter().find(|l| l.name == address.suburb) {
            Some(l) => l,
            None => {
                eprintln!("Suburb {} not found", address.suburb);
                return;
            }
        };
        if cli.debug {
            println!("{:?}", my_locality);
        }

        let streets = match client.get_streets(&my_locality.id).await {
            Ok(val) => val,
            Err(e) => {
                eprintln!("Failed to query streets: {}", e);
                return;
            }
        };

        let my_street: Street = match streets.into_iter().find(|s| s.name == address.street) {
            Some(s) => s,
            None => {
                eprintln!("Street '{}' not found", address.street);
                return;
            }
        };
        let properties = match client.get_properties(&my_street.id).await {
            Ok(val) => val,
            Err(e) => {
                eprintln!("Failed to query properties: {}", e);
                return;
            }
        };

        let my_property = match properties
            .iter()
            .find(|p| p.name.contains(&address.address))
        {
            Some(p) => p,
            None => {
                eprintln!("Property {} not found", address.address);
                return;
            }
        };
        if cli.debug {
            println!("{:?}", my_property);
        }

        match client.get_property(&my_property.id, cli.debug).await {
            Ok(val) => val,
            Err(e) => {
                eprintln!("{}", e);
                return;
            }
        }
    };

    let (bindata, bin_days) = (bin_data.0.clone(), bin_data.1.clone());
    let mut days = HashSet::new();

    bindata.dow.iter().for_each(|d| {
        days.insert(d);
    });
    bindata.days_of_week.iter().for_each(|d| {
        days.insert(d);
    });

    if cli.show_day {
        for day in days {
            println!("{}", day);
        }
    } else if cli.pretty {
        println!("{}", bindata.property);

        let mut bin_days_sorted = bin_days
            .iter()
            .filter(|day| {
                if cli.future {
                    match day.get_start_date() {
                        Ok(val) => {
                            if val < time::OffsetDateTime::now_utc().date() {
                                return false;
                            }
                        }
                        Err(_) => return false,
                    }
                }
                true
            })
            .collect::<Vec<&BinDay>>();

        bin_days_sorted.sort_by_key(|f| {
            f.get_start_date()
                .unwrap_or(time::OffsetDateTime::now_utc().date())
        });
        for day in bin_days_sorted {
            if cli.future {
                match day.get_start_date() {
                    Ok(val) => {
                        if val < time::OffsetDateTime::now_utc().date() {
                            continue;
                        }
                    }
                    Err(_) => continue,
                }
                if day.get_start_date().is_err() {
                    continue;
                }
            }
            println!("- {}", day);
        }
    } else {
        println!(
            "{}",
            serde_json::to_string_pretty(&bin_data).expect("Failed to serialize bin data")
        );
    }
}
