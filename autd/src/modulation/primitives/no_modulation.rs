/*
 * File: no_modulation.rs
 * Project: primitives
 * Created Date: 22/05/2020
 * Author: Shun Suzuki
 * -----
 * Last Modified: 07/08/2020
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2020 Hapis Lab. All rights reserved.
 *
 */

use super::super::Modulation;

/// Static amplitude.
pub struct NoModulation {}

impl NoModulation {
    pub fn create(amp: u8) -> Modulation {
        Modulation::new(vec![amp; 2])
    }
}
