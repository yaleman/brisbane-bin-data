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

use brisbane_bin_data::{cli::Cli, AddressData};
use clap::Parser;
use serde_json::json;
use std::collections::HashSet;

use brisbane_bin_data::{BinClient, BinDay};

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let mut client = BinClient::default();
    if cli.debug {
        client = client.with_debug();
    }

    let bin_data = if let Some(property_id) = cli.property_id {
        match client.get_property(property_id).await {
            Ok(val) => val,
            Err(e) => {
                eprintln!("{e}");
                return;
            }
        }
    } else {
        let address: AddressData = match cli.get_data() {
            Ok(val) => val,
            Err(e) => {
                eprintln!("{e}");
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
        match client.get_address(address).await {
            Ok(val) => val,
            Err(e) => {
                eprintln!("{e}");
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
            println!("{day}");
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
            println!("- {day}");
        }
    } else {
        println!("{}", json!(&bin_data));
    }
}
