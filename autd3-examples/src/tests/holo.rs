/*
 * File: holo.rs
 * Project: tests
 * Created Date: 29/05/2021
 * Author: Shun Suzuki
 * -----
 * Last Modified: 16/12/2021
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2021 Hapis Lab. All rights reserved.
 *
 */

use anyhow::Result;
use autd3::prelude::*;
use autd3_holo_gain::*;
use colored::*;
use std::io::{self, Write};

pub async fn holo<L: Link>(mut autd: Controller<L>) -> Result<Controller<L>> {
    autd.silent_mode = true;

    let mut m = Sine::new(150);

    let center = Vector3::new(
        TRANS_SPACING_MM * ((NUM_TRANS_X - 1) as f64 / 2.0),
        TRANS_SPACING_MM * ((NUM_TRANS_Y - 1) as f64 / 2.0),
        150.0,
    );

    let foci = vec![
        center + Vector3::new(20., 0., 0.),
        center + Vector3::new(-20., 0., 0.),
    ];
    let amps = vec![1., 1.];

    println!("[0]: SDP");
    println!("[1]: EVD");
    println!("[2]: Naive");
    println!("[3]: GS");
    println!("[4]: GS-PAT");
    println!("[5]: LM");
    println!("[6]: Greedy");
    println!("[Others]: SDP");
    print!("{}", "Choose number: ".green().bold());
    io::stdout().flush()?;

    let mut s = String::new();
    io::stdin().read_line(&mut s)?;
    match s.trim().parse::<usize>() {
        Ok(0) => {
            let mut g = Sdp::<NalgebraBackend>::new(foci, amps);
            autd.send(&mut m, &mut g).await?;
        }
        Ok(1) => {
            let mut g = Evd::<NalgebraBackend>::new(foci, amps);
            autd.send(&mut m, &mut g).await?;
        }
        Ok(2) => {
            let mut g = Naive::<NalgebraBackend>::new(foci, amps);
            autd.send(&mut m, &mut g).await?;
        }
        Ok(3) => {
            let mut g = Gs::<NalgebraBackend>::new(foci, amps);
            autd.send(&mut m, &mut g).await?;
        }
        Ok(4) => {
            let mut g = GsPat::<NalgebraBackend>::new(foci, amps);
            autd.send(&mut m, &mut g).await?;
        }
        Ok(5) => {
            let mut g = Lm::<NalgebraBackend>::new(foci, amps);
            autd.send(&mut m, &mut g).await?;
        }
        Ok(6) => {
            let mut g = Greedy::new(foci, amps);
            autd.send(&mut m, &mut g).await?;
        }
        _ => {
            let mut g = Sdp::<NalgebraBackend>::new(foci, amps);
            autd.send(&mut m, &mut g).await?;
        }
    };

    Ok(autd)
}
