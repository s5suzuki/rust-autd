/*
 * File: test_runner.rs
 * Project: src
 * Created Date: 28/05/2021
 * Author: Shun Suzuki
 * -----
 * Last Modified: 03/10/2021
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2021 Hapis Lab. All rights reserved.
 *
 */

extern crate colored;

use crate::tests::*;
use anyhow::Result;
use autd3::prelude::*;
use colored::*;
use std::io::{self, Write};

pub async fn run<L: Link>(autd: Controller<L>) -> Result<()> {
    let mut autd = autd;

    println!("***** Firmware information *****");
    let firm_list = autd.firmware_infos().await?;
    for firm_info in firm_list {
        println!("{}", firm_info);
    }
    println!("********************************");

    autd.clear().await?;

    loop {
        println!("[0]: Single Focal Point Test");
        println!("[1]: BesselBeam Test");
        println!("[2]: Multiple foci Test");
        println!("[3]: Spatio-Temporal Modulation Test");
        println!("[4]: PointSequence (hardware STM) Test");
        println!("[5]: GainSequence (hardware STM with arbitrary Gain) Test");
        println!("[Others]: Finish");
        print!("{}", "Choose number: ".green().bold());
        io::stdout().flush()?;

        let mut s = String::new();
        io::stdin().read_line(&mut s)?;
        autd = match s.trim().parse::<usize>() {
            Ok(0) => simple(autd).await?,
            Ok(1) => bessel(autd).await?,
            Ok(2) => holo(autd).await?,
            Ok(3) => stm(autd).await?,
            Ok(4) => seq(autd).await?,
            Ok(5) => seq_gain(autd).await?,
            _ => break,
        };

        println!("press any key to finish...");
        let mut _s = String::new();
        io::stdin().read_line(&mut _s)?;

        autd.stop().await?;
        println!("finish");
    }

    Ok(())
}
