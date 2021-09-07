/*
 * File: seq.rs
 * Project: tests
 * Created Date: 28/05/2021
 * Author: Shun Suzuki
 * -----
 * Last Modified: 07/09/2021
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2021 Hapis Lab. All rights reserved.
 *
 */

use anyhow::Result;
use autd3::prelude::*;

pub async fn seq<L: Link>(mut autd: Controller<L>) -> Result<Controller<L>> {
    autd.silent_mode = false;

    let mut m = Static::new();
    autd.send_modulation(&mut m).await?;

    let mut seq = PointSequence::new();

    let center = Vector3::new(
        TRANS_SPACING_MM * ((NUM_TRANS_X - 1) as f64 / 2.0),
        TRANS_SPACING_MM * ((NUM_TRANS_Y - 1) as f64 / 2.0),
        150.0,
    );

    let point_num = 200;
    for i in 0..point_num {
        let radius = 30.0;
        let theta = 2.0 * std::f64::consts::PI * i as f64 / point_num as f64;
        let p = radius * Vector3::new(theta.cos(), theta.sin(), 0.0);
        seq.add_point(center + p, 0xFF)?;
    }

    let f = seq.set_freq(1.0);
    println!("Actual frequency is {} Hz", f);
    autd.send_seq(&mut seq).await?;

    Ok(autd)
}
