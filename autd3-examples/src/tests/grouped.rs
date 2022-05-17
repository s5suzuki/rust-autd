/*
 * File: grouped.rs
 * Project: tests
 * Created Date: 13/05/2022
 * Author: Shun Suzuki
 * -----
 * Last Modified: 13/05/2022
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2022 Hapis Lab. All rights reserved.
 *
 */

#[macro_export]
macro_rules! grouped {
    ($autd:ident) => {{
        let silencer_config = SilencerConfig::default();
        $autd.config_silencer(silencer_config)?;

        let g1 = Focus::new($autd.geometry().devices()[0].center() + Vector3::new(0., 0., 150.0));
        let g2 = Bessel::new(
            $autd.geometry().devices()[1].center(),
            Vector3::z(),
            18. / 180. * std::f64::consts::PI,
        );

        let mut g = Grouped::new();
        g.add(0, g1);
        g.add(1, g2);

        let mut m = Sine::new(150);

        $autd.send(&mut m).send(&mut g)?;
    }};
}