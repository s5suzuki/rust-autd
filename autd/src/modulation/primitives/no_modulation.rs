/*
 * File: no_modulation.rs
 * Project: primitives
 * Created Date: 22/05/2020
 * Author: Shun Suzuki
 * -----
 * Last Modified: 30/12/2020
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2020 Hapis Lab. All rights reserved.
 *
 */

use super::super::Modulation;
use crate::core::configuration::Configuration;

/// Static amplitude.
pub struct NoModulation {
    buffer: [u8; 1],
    sent: usize,
}

impl NoModulation {
    pub fn create(amp: u8) -> Self {
        Self {
            buffer: [amp; 1],
            sent: 0,
        }
    }
}

impl Modulation for NoModulation {
    fn build(&mut self, _config: Configuration) {}

    fn buffer(&self) -> &[u8] {
        &self.buffer
    }

    fn sent(&mut self) -> &mut usize {
        &mut self.sent
    }
}
