/*
 * File: simple_test.rs
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

use autd::prelude::*;

pub fn simple_test<L: Link>(autd: &mut AUTD<L>) -> Result<(), Box<dyn Error>> {
    let mut g = FocalPointGain::create(Vector3::new(90., 70., 150.));
    autd.append_gain_sync(&mut g)?;

    let mut m = SineModulation::create(150);
    autd.append_modulation_sync(&mut m)?;

    Ok(())
}
