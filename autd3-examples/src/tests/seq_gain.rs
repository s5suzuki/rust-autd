/*
 * File: seq.rs
 * Project: tests
 * Created Date: 28/05/2021
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

pub async fn seq_gain<L: Link>(mut autd: Controller<L>) -> Result<Controller<L>> {
    autd.silent_mode = false;

    let mut m = Static::new();

    let mut seq = GainSequence::new();

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

        let g = Sdp::<NalgebraBackend>::new(vec![center + p, center - p], vec![1.0, 1.0]);
        seq.add_gain(g, autd.geometry())?;
    }

    let f = seq.set_freq(1.0);
    println!("Actual frequency is {} Hz", f);
    autd.send(&mut m, &mut seq).await?;

    Ok(autd)
}
