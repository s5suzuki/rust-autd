/*
 * File: soft_stm_test.rs
 * Project: example
 * Created Date: 12/12/2019
 * Author: Shun Suzuki
 * -----
 * Last Modified: 25/05/2020
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2019 Hapis Lab. All rights reserved.
 *
 */

use std::f64::consts::PI;

use std::error::Error;

use autd::prelude::*;

pub fn soft_stm_test(autd: &mut AUTD) -> Result<(), Box<dyn Error>> {
    let m = NoModulation::create(255);
    autd.append_modulation_sync(m);

    let mut gains: Vec<Box<dyn Gain>> = Vec::new();
    let center = Vector3::new(90., 70., 150.);
    let r = 30.;
    for i in 0..200 {
        let theta = 2. * PI * i as f64 / 200.;
        let p = Vector3::new(r * theta.cos(), r * theta.sin(), 0.);
        gains.push(FocalPointGain::create(center + p));
    }
    autd.append_stm_gains(gains);
    autd.start_stm(1.);

    Ok(())
}
