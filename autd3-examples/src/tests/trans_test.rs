/*
 * File: trans_test.rs
 * Project: tests
 * Created Date: 09/05/2022
 * Author: Shun Suzuki
 * -----
 * Last Modified: 09/05/2022
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2022 Hapis Lab. All rights reserved.
 *
 */

#[macro_export]
macro_rules! trans_test {
    ($autd:ident) => {{
        let silencer_config = SilencerConfig::default();
        $autd.config_silencer(silencer_config)?;

        let mut g = TransducerTest::new();
        g.set(0, 0., 1.0);
        g.set(17, 0., 1.0);
        g.set(NUM_TRANS_IN_UNIT * 2 + 0, 0., 1.0);
        g.set(NUM_TRANS_IN_UNIT * 2 + 17, 0., 1.0);

        let mut m = Static::new(0xFF);

        $autd.send(&mut m).send(&mut g)?;
    }};
}
