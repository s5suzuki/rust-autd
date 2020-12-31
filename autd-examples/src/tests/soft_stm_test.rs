/*
 * File: soft_stm_test.rs
 * Project: example
 * Created Date: 12/12/2019
 * Author: Shun Suzuki
 * -----
 * Last Modified: 31/12/2020
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2019 Hapis Lab. All rights reserved.
 *
 */

use std::error::Error;

use autd::{prelude::*, Float, PI};

pub fn soft_stm_test<L: Link>(autd: &mut AUTD<L>) -> Result<(), Box<dyn Error>> {
    let mut m = NoModulation::create(255);
    autd.append_modulation_sync(&mut m)?;

    let center = Vector3::new(90., 70., 150.);
    let r = 30.;
    for i in 0..200 {
        let theta = 2. * PI * i as Float / 200.;
        let p = Vector3::new(r * theta.cos(), r * theta.sin(), 0.);
        autd.append_stm_gain(FocalPointGain::create(center + p));
    }
    autd.start_stm(1.);

    Ok(())
}
