/*
 * File: static.rs
 * Project: modulation
 * Created Date: 27/05/2021
 * Author: Shun Suzuki
 * -----
 * Last Modified: 30/05/2021
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2021 Hapis Lab. All rights reserved.
 *
 */

use anyhow::Result;
use autd3_core::{
    configuration::Configuration, hardware_defined::MOD_FRAME_SIZE, modulation::Modulation,
};
use autd3_traits::Modulation;

/// Static amplitude
#[derive(Modulation)]
pub struct Static {
    buffer: Vec<u8>,
    sent: usize,
}

impl Static {
    /// constructor
    pub fn new() -> Self {
        Self::with_duty(0xFF)
    }

    /// constructor
    ///
    /// # Arguments
    ///
    /// * `duty` - duty ratio
    ///
    pub fn with_duty(duty: u8) -> Self {
        Self {
            buffer: vec![duty; MOD_FRAME_SIZE],
            sent: 0,
        }
    }

    #[allow(clippy::unnecessary_wraps)]
    fn calc(&mut self, _config: Configuration) -> Result<()> {
        Ok(())
    }
}

impl Default for Static {
    fn default() -> Self {
        Self::new()
    }
}
