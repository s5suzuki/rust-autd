/*
 * File: sine_pressure.rs
 * Project: modulation
 * Created Date: 05/06/2021
 * Author: Shun Suzuki
 * -----
 * Last Modified: 05/06/2021
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2021 Hapis Lab. All rights reserved.
 *
 */

use std::f64::consts::PI;

use anyhow::Result;
use autd3_core::{configuration::Configuration, modulation::Modulation};
use autd3_traits::Modulation;

use num::integer::gcd;

/// Sine wave modulation in radiation pressure
#[derive(Modulation)]
pub struct SinePressure {
    buffer: Vec<u8>,
    sent: usize,
    freq: usize,
    amp: f64,
    offset: f64,
}

impl SinePressure {
    /// constructor.
    ///
    /// # Arguments
    ///
    /// * `freq` - Frequency of the sine wave
    ///
    pub fn new(freq: usize) -> Self {
        Self::with_params(freq, 1.0, 0.5)
    }

    /// constructor.
    /// Sine wave oscillate from `offset`-`amp`/2 to `offset`+`amp`/2
    ///
    /// # Arguments
    ///
    /// * `freq` - Frequency of the sine wave
    /// * `amp` - peek to peek amplitude of the wave (Maximum value is 1.0)
    /// * `offset` - Offset of the wave
    ///
    pub fn with_params(freq: usize, amp: f64, offset: f64) -> Self {
        Self {
            buffer: vec![],
            sent: 0,
            freq,
            amp,
            offset,
        }
    }

    #[allow(clippy::unnecessary_wraps)]
    fn calc(&mut self, config: Configuration) -> Result<()> {
        let sf = config.mod_sampling_frequency() as usize;
        let mod_buf_size = config.mod_buf_size() as usize;

        let freq = self.freq.clamp(1, sf / 2);

        let d = gcd(sf, freq);

        let n = mod_buf_size / d / (mod_buf_size / sf);
        let rep = freq / d;

        self.buffer.resize(n, 0);

        for i in 0..n {
            let amp = self.amp / 2.0 * (2.0 * PI * (rep * i) as f64 / n as f64).sin() + self.offset;
            let amp = amp.sqrt().clamp(0.0, 1.0);
            let duty = amp.asin() * 2.0 / PI * 255.0;
            self.buffer[i] = duty as u8;
        }

        Ok(())
    }
}
