/*
 * File: two_autd.rs
 * Project: src
 * Created Date: 25/05/2020
 * Author: Shun Suzuki
 * -----
 * Last Modified: 25/05/2020
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2020 Hapis Lab. All rights reserved.
 *
 */

extern crate colored;

mod tests;

use autd::prelude::*;
use autd_soem_link::{EthernetAdapters, SoemLink};
use colored::*;
use std::io;
use std::io::Write;
use tests::*;

fn get_adapter() -> String {
    let adapters: EthernetAdapters = Default::default();
    for (index, adapter) in adapters.into_iter().enumerate() {
        println!("[{}]: {}", index, adapter);
    }

    let i: usize;
    loop {
        print!("{}", "Choose number: ".green().bold());
        io::stdout().flush().unwrap();

        let mut s = String::new();
        io::stdin().read_line(&mut s).unwrap();
        if let Ok(num) = s.trim().parse() {
            if num < adapters.len() {
                i = num;
                break;
            }
        }
    }
    let adapter = &adapters[i];
    adapter.name.to_string()
}

fn main() {
    let mut autd = AUTD::create();

    autd.geometry()
        .add_device(Vector3::zeros(), Vector3::zeros());
    autd.geometry()
        .add_device(Vector3::zeros(), Vector3::zeros());

    let ifname = get_adapter();
    let link = SoemLink::new(&ifname, autd.geometry().num_devices() as u16);

    autd.open(link).expect("Failed to open");

    run(autd);
}
