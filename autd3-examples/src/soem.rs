/*
 * File: soem.rs
 * Project: src
 * Created Date: 28/05/2021
 * Author: Shun Suzuki
 * -----
 * Last Modified: 28/05/2021
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2021 Hapis Lab. All rights reserved.
 *
 */

mod test_runner;
mod tests;

use autd3::prelude::*;
use autd3_soem_link::{EthernetAdapters, SoemLink};
use colored::*;
use std::io::{self, Write};
use test_runner::run;

fn get_adapter() -> String {
    let adapters: EthernetAdapters = Default::default();
    for (index, adapter) in adapters.into_iter().enumerate() {
        println!("[{}]: {}", index, adapter);
    }

    let i: usize;
    let mut s = String::new();
    loop {
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

async fn main_task() {
    let mut geometry = Geometry::new();
    geometry.add_device(Vector3::zeros(), Vector3::zeros());

    let ifname = get_adapter();
    let link = SoemLink::new(&ifname, geometry.num_devices() as u16, 1);

    let autd = Controller::open(geometry, link).expect("Failed to open");

    run(autd).await.expect("Some error occurred.");
}

fn main() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async { main_task().await });
}
