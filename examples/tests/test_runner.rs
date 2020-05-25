/*
 * File: test_runner.rs
 * Project: autd
 * Created Date: 29/08/2019
 * Author: Shun Suzuki
 * -----
 * Last Modified: 25/05/2020
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2019 Hapis Lab. All rights reserved.
 *
 */

extern crate colored;

use std::error::Error;
use std::io;
use std::io::Write;

use crate::tests::*;
use autd::prelude::*;
use colored::*;

type TestFn = fn(&mut AUTD) -> Result<(), Box<dyn Error>>;

pub fn run(mut autd: AUTD) {
    println!("***** Firmware information *****");
    let firm_list = autd.firmware_info_list();
    for firm_info in firm_list {
        println!("{}", firm_info);
    }
    println!("********************************");

    let examples: Vec<(TestFn, _)> = vec![
        (simple_test, "Single Focal Point Test"),
        (bessel_test, "BesselBeam Test"),
        (soft_stm_test, "Spatio-temporal Modulation Test"),
    ];

    loop {
        for (i, (_, desc)) in examples.iter().enumerate() {
            println!("[{}]: {}", i, desc);
        }
        println!("[Others]: Finish");
        println!(
            "{}",
            "Make sure you connected ONLY appropriate numbers of AUTD."
                .yellow()
                .bold()
        );

        print!("{}", "Choose number: ".green().bold());
        io::stdout().flush().unwrap();

        let mut s = String::new();
        io::stdin().read_line(&mut s).unwrap();
        let i: usize = match s.trim().parse() {
            Ok(num) if num < examples.len() => num,
            _ => break,
        };

        let (f, _) = examples[i];

        match f(&mut autd) {
            Ok(_) => {
                println!("press any key to finish...");
                let mut input = String::new();
                io::stdin().read_line(&mut input).unwrap();
                autd.stop();
                println!("finish");
            }
            Err(e) => {
                eprintln!("{}", e.to_string().red().bold());
            }
        }
    }
}
