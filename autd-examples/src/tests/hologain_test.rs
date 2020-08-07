/*
 * File: hologain_test.rs
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

use std::error::Error;

use autd::prelude::*;
use autd_holo_gain::HoloGain;

pub fn hologain_test(autd: &mut AUTD) -> Result<(), Box<dyn Error>> {
    let g = HoloGain::create(
        vec![Vector3::new(70., 70., 150.), Vector3::new(110., 70., 150.)],
        vec![1., 1.],
    );
    autd.append_gain_sync(g);

    let m = SineModulation::create(150);
    autd.append_modulation_sync(m);
    Ok(())
}
