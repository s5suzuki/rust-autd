/*
 * File: bessel.rs
 * Project: tests
 * Created Date: 28/05/2021
 * Author: Shun Suzuki
 * -----
 * Last Modified: 06/05/2022
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2021 Hapis Lab. All rights reserved.
 *
 */

#[macro_export]
macro_rules! bessel {
    ($autd:ident) => {{
        let silencer_config = SilencerConfig::default();
        $autd.config_silencer(silencer_config)?;

        let center = $autd.geometry().center();
        let dir = Vector3::z();

        let mut g = Bessel::new(center, dir, 18. / 180. * std::f64::consts::PI);
        let mut m = Sine::new(150);

        $autd.send(&mut m).send(&mut g)?;
    }};
}
