/*
 * File: stm.rs
 * Project: tests
 * Created Date: 28/05/2021
 * Author: Shun Suzuki
 * -----
 * Last Modified: 29/05/2021
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2021 Hapis Lab. All rights reserved.
 *
 */

use anyhow::Result;
use autd3::prelude::*;
use std::io;

pub async fn stm<L: Link>(mut autd: Controller<L>) -> Result<Controller<L>> {
    autd.silent_mode = false;

    let mut m = Static::new();
    autd.send_modulation(&mut m).await?;

    let mut stm = autd.stm();

    let center = Vector3::new(
        TRANS_SPACING_MM * ((NUM_TRANS_X - 1) as f64 / 2.0),
        TRANS_SPACING_MM * ((NUM_TRANS_Y - 1) as f64 / 2.0),
        150.0,
    );
    let point_num = 100;
    for i in 0..point_num {
        let radius = 20.0;
        let theta = 2.0 * std::f64::consts::PI * i as f64 / point_num as f64;
        let pos = radius * Vector3::new(theta.cos(), theta.sin(), 0.0);
        let mut g = Focus::new(center + pos);
        stm.add(&mut g)?;
    }

    let timer = stm.start(0.5)?; // 0.5 Hz

    println!("press any key to stop STM...");
    let mut _s = String::new();
    io::stdin().read_line(&mut _s)?;

    let mut stm = timer.stop()?;
    stm.finish();

    let autd = stm.controller();

    Ok(autd)
}
