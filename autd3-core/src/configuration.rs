/*
 * File: configuration.rs
 * Project: src
 * Created Date: 24/05/2021
 * Author: Shun Suzuki
 * -----
 * Last Modified: 30/05/2021
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2021 Hapis Lab. All rights reserved.
 *
 */

use crate::hardware_defined::{MOD_BUF_SIZE_MAX, MOD_SAMPLING_FREQ_BASE};

/// Configuration of modulation
#[derive(Clone, Copy)]
pub struct Configuration {
    mod_smpl_freq_div: u16,
    mod_buf_size: u16,
}

impl Configuration {
    /// Construct configuration
    ///
    /// Modulation is sampled and repeated from a buffer of size `mod_buf_size` at a sampling rate of [MOD_SAMPLING_FREQ_BASE](crate::hardware_defined::MOD_SAMPLING_FREQ_BASE)/`mod_smpl_freq_div`.
    ///
    /// # Arguments
    ///
    /// * `mod_smpl_freq_div` - Modulation sampling frequency division
    /// * `mod_buf_size` - Modulation buffer size
    ///
    pub fn new(mod_smpl_freq_div: u16, mod_buf_size: u16) -> Self {
        let mod_smpl_freq_div = mod_smpl_freq_div.max(1);
        let mod_buf_size = mod_buf_size.min(MOD_BUF_SIZE_MAX);
        Self {
            mod_smpl_freq_div,
            mod_buf_size,
        }
    }

    /// Return [MOD_SAMPLING_FREQ_BASE](crate::hardware_defined::MOD_SAMPLING_FREQ_BASE)/`mod_smpl_freq_div`
    pub fn mod_sampling_frequency(&self) -> f64 {
        MOD_SAMPLING_FREQ_BASE / self.mod_smpl_freq_div as f64
    }

    pub fn mod_sampling_frequency_division(&self) -> u16 {
        self.mod_smpl_freq_div
    }

    pub fn mod_buf_size(&self) -> u16 {
        self.mod_buf_size
    }
}

impl Default for Configuration {
    fn default() -> Self {
        Self::new(10, 4000)
    }
}
