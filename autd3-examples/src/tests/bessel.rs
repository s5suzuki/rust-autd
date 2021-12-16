/*
 * File: bessel.rs
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

pub async fn bessel<L: Link>(mut autd: Controller<L>) -> Result<Controller<L>> {
    autd.silent_mode = true;

    let center = Vector3::new(
        TRANS_SPACING_MM * ((NUM_TRANS_X - 1) as f64 / 2.0),
        TRANS_SPACING_MM * ((NUM_TRANS_Y - 1) as f64 / 2.0),
        0.0,
    );

    let mut g = Bessel::new(center, Vector3::z(), 18. / 180. * std::f64::consts::PI);
    let mut m = Sine::new(150);

    autd.send(&mut m, &mut g).await?;

    Ok(autd)
}
