/*
 * File: groupedgain_test.rs
 * Project: example
 * Created Date: 12/12/2019
 * Author: Shun Suzuki
 * -----
 * Last Modified: 07/08/2020
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2019 Hapis Lab. All rights reserved.
 *
 */

use std::collections::HashMap;
use std::error::Error;

use autd::prelude::*;
use std::f64::consts::PI;

pub fn grouped_gain_test(autd: &mut AUTD) -> Result<(), Box<dyn Error>> {
    let g1 = FocalPointGain::create(Vector3::new(90., 70., 200.));
    let g2 = BesselBeamGain::create(
        Vector3::new(90., 70., 200.),
        Vector3::z(),
        18.0 / 180.0 * PI,
    );
    let mut ids = HashMap::new();
    // Any type of key which implements "Sized + Send + Hash + Eq" can be used
    ids.insert("A", vec![0]); // Group "A" consists of devices with id: 0,...
    ids.insert("B", vec![1]); // Group "B" consists of devices with id: 1,...
    let mut gains: HashMap<_, Box<dyn Gain>> = HashMap::new();
    gains.insert("A", g1);
    gains.insert("B", g2);
    let g = GroupedGain::create(ids, gains);
    autd.append_gain_sync(g);

    let m = SineModulation::create(150);
    autd.append_modulation_sync(m);
    Ok(())
}
