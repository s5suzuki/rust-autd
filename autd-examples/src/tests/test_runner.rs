/*
 * File: test_runner.rs
 * Project: autd
 * Created Date: 29/08/2019
 * Author: Shun Suzuki
 * -----
 * Last Modified: 31/12/2020
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

type TestFn<L> = fn(&mut AUTD<L>) -> Result<(), Box<dyn Error>>;

pub fn run<L: Link>(mut autd: AUTD<L>) -> Result<(), Box<dyn Error>> {
    autd.clear()?;

    println!("***** Firmware information *****");
    let firm_list = autd.firmware_info_list()?;
    for firm_info in firm_list {
        println!("{}", firm_info);
    }
    println!("********************************");

    println!(
        "{}",
        "Make sure you connected ONLY appropriate numbers of AUTD."
            .yellow()
            .bold()
    );

    #[allow(unused_mut)]
    let mut examples: Vec<(TestFn<L>, _)> = vec![
        (simple_test, "Single Focal Point Test"),
        (bessel_test, "BesselBeam Test"),
        (soft_stm_test, "Spatio-temporal Modulation Test"),
        (point_sequence_test, "Point Sequence Test (Hardware STM)"),
    ];

    #[cfg(feature = "hologain")]
    {
        examples.push((hologain_test, "HoloGain Test (2 focal points)"));
    }

    #[cfg(feature = "csvgain")]
    {
        examples.push((csvgain_test, "CsvGain Test"));
    }

    #[cfg(feature = "wavmodulation")]
    {
        examples.push((wav_modulation_test, "WavModulation Test"));
    }

    examples.push((
        grouped_gain_test,
        "Grouped Gain Test (2 AUTDs are required)",
    ));

    loop {
        for (i, (_, desc)) in examples.iter().enumerate() {
            println!("[{}]: {}", i, desc);
        }
        println!("[Others]: Finish");

        print!("{}", "Choose number: ".green().bold());
        io::stdout().flush()?;

        let mut s = String::new();
        io::stdin().read_line(&mut s)?;
        let i: usize = match s.trim().parse() {
            Ok(num) if num < examples.len() => num,
            _ => break,
        };

        let (f, _) = examples[i];

        match f(&mut autd) {
            Ok(_) => {
                println!("press any key to finish...");
                let mut input = String::new();
                io::stdin().read_line(&mut input)?;
                autd.stop()?;
                autd.clear()?;
                println!("finish");
            }
            Err(e) => {
                eprintln!("{}", e.to_string().red().bold());
            }
        }
    }

    autd.clear()?;
    autd.close()?;

    Ok(())
}
