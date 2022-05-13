/*
 * File: soem.rs
 * Project: src
 * Created Date: 27/04/2022
 * Author: Shun Suzuki
 * -----
 * Last Modified: 13/05/2022
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2022 Hapis Lab. All rights reserved.
 *
 */

mod test_runner;
mod tests;

use anyhow::Result;

use std::io::{self, Write};

use colored::*;

use autd3::prelude::*;
use autd3_link_soem::{Config, EthernetAdapters, SOEM};

fn get_adapter() -> String {
    let adapters: EthernetAdapters = Default::default();
    adapters
        .into_iter()
        .enumerate()
        .for_each(|(index, adapter)| {
            println!("[{}]: {}", index, adapter);
        });

    let i: usize;
    loop {
        let mut s = String::new();
        print!("{}", "Choose number: ".green().bold());
        io::stdout().flush().unwrap();

        io::stdin().read_line(&mut s).unwrap();
        match s.trim().parse() {
            Ok(num) if num < adapters.len() => {
                i = num;
                break;
            }
            _ => continue,
        };
    }
    let adapter = &adapters[i];
    adapter.name.to_string()
}

fn main() -> Result<()> {
    let mut geometry = GeometryBuilder::new().legacy_mode().build();
    geometry.add_device(Vector3::zeros(), Vector3::zeros());
    // let mut geometry = GeometryBuilder::new().build();
    // geometry.add_device(Vector3::zeros(), Vector3::zeros());
    // geometry
    //     .transducers_mut()
    //     .for_each(|t| t.set_frequency(40e3).unwrap());

    let ifname = get_adapter();
    let config = Config {
        cycle_ticks: 4,
        high_precision_timer: true,
    };
    let link = SOEM::new(&ifname, geometry.num_devices() as u16, config, |msg| {
        eprintln!("unrecoverable error occurred: {}", msg);
        std::process::exit(-1);
    });

    let autd = Controller::open(geometry, link).expect("Failed to open");

    run!(autd);

    Ok(())
}
