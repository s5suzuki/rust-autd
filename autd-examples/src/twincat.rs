/*
 * File: twincat.rs
 * Project: examples
 * Created Date: 25/05/2020
 * Author: Shun Suzuki
 * -----
 * Last Modified: 25/05/2020
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2020 Hapis Lab. All rights reserved.
 *
 */

mod tests;

use autd::prelude::*;
use autd_twincat_link::LocalTwinCATLink;
use tests::*;

fn main() {
    let mut autd = AUTD::create();

    autd.geometry()
        .add_device(Vector3::zeros(), Vector3::zeros());

    let link = LocalTwinCATLink::new();

    autd.open(link).expect("Failed to open");

    run(autd);
}
