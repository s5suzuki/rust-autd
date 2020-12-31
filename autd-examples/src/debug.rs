/*
 * File: debug.rs
 * Project: src
 * Created Date: 31/12/2020
 * Author: Shun Suzuki
 * -----
 * Last Modified: 31/12/2020
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2020 Hapis Lab. All rights reserved.
 *
 */

mod tests;

use autd::{link::debug_link::DebugLink, prelude::*};
use std::fs::File;
use tests::*;

fn main() {
    let mut geometry = Geometry::new();
    geometry.add_device(Vector3::zeros(), Vector3::zeros());

    let out = File::create("log.txt").unwrap();
    let link = DebugLink::new(out);

    let autd = AUTD::open(geometry, link).expect("Failed to open");

    run(autd).expect("Some error occurred.");
}
