/*
 * File: main.rs
 * Project: examples
 * Created Date: 29/06/2020
 * Author: Shun Suzuki
 * -----
 * Last Modified: 04/07/2020
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2020 Hapis Lab. All rights reserved.
 *
 */

use autd::prelude::*;
use autd_sequence::primitives::CircumSeq;
use autd_soem_link::{EthernetAdapters, SoemLink};
use std::io;
use std::io::Write;

fn get_adapter() -> String {
    let adapters: EthernetAdapters = Default::default();
    for (index, adapter) in adapters.into_iter().enumerate() {
        println!("[{}]: {}", index, adapter);
    }

    let i: usize;
    loop {
        print!("Choose number: ");
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

    let ifname = get_adapter();
    let link = SoemLink::new(&ifname, autd.geometry().num_devices() as u16);

    autd.open(link).expect("Failed to open");
    let success = autd.clear().unwrap();
    if !success {
        println!("Failed to clear");
    }

    let success = autd.calibrate().unwrap();
    if !success {
        println!("Failed to calibrate");
    }

    println!("***** Firmware information *****");
    let firm_list = autd.firmware_info_list();
    for firm_info in firm_list {
        println!("{}", firm_info);
    }
    println!("********************************");

    // let g = FocalPointGain::create(Vector3::new(90., 70., 150.));
    // autd.append_gain_sync(g);

    // let m = SineModulation::create(150);
    // autd.append_modulation_sync(m);

    autd.set_silent_mode(false);
    let mut seq = CircumSeq::create(
        Vector3::new(AUTD_WIDTH / 2.0, AUTD_HEIGHT / 2.0, 150.),
        Vector3::z(),
        30.0,
        200,
    );
    let actual_freq = seq.set_freq(200.);
    println!("actual frequency: {}", actual_freq);
    autd.append_sequence(seq);

    println!("press enter key to finish...");
    let mut s = String::new();
    io::stdin().read_line(&mut s).unwrap();

    autd.clear().expect("Failed to clear");
    autd.close();
}
