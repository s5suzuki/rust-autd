/*
 * File: point_sequence.rs
 * Project: tests
 * Created Date: 30/06/2020
 * Author: Shun Suzuki
 * -----
 * Last Modified: 30/06/2020
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2020 Hapis Lab. All rights reserved.
 *
 */

use std::error::Error;

use autd::prelude::*;

pub fn point_sequence_test(autd: &mut AUTD) -> Result<(), Box<dyn Error>> {
    autd.set_silent_mode(false);
    let mut seq = CircumSeq::create(
        Vector3::new(AUTD_WIDTH / 2.0, AUTD_HEIGHT / 2.0, 150.),
        Vector3::z(),
        30.0,
        200,
    );
    let actual_freq = seq.set_freq(200.);
    println!("actual frequency: {}", actual_freq);
    autd.append_sequence(seq);

    Ok(())
}
