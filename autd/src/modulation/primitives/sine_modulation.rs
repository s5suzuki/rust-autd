/*
 * File: sine_modulation.rs
 * Project: modulation
 * Created Date: 16/11/2019
 * Author: Shun Suzuki
 * -----
 * Last Modified: 30/12/2020
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2019 Hapis Lab. All rights reserved.
 *
 */

use super::super::Modulation;
use crate::core::configuration::Configuration;
use crate::Float;

use num::clamp;
use num::integer::gcd;

/// Sine wave modulation
pub struct SineModulation {
    freq: i32,
    amp: Float,
    offset: Float,
    buffer: Vec<u8>,
    sent: usize,
}

impl SineModulation {
    /// Generate SineModulation.
    /// The sine wave oscillate from offset-amp/2 to offset+amp/2
    ///
    /// # Arguments
    ///
    /// * `freq` - Frequency of the sine wave.
    /// * `amp` - peek to peek amplitude of the wave (Maximum value is 1.0).
    /// * `offset` - Offset of the wave.
    ///
    pub fn create_with_amp(freq: i32, amp: Float, offset: Float) -> Self {
        Self {
            freq,
            amp,
            offset,
            buffer: vec![],
            sent: 0,
        }
    }

    /// Generate SineModulation.
    ///
    /// # Arguments
    ///
    /// * `freq` - Frequency of the sine wave.
    ///
    pub fn create(freq: i32) -> Self {
        SineModulation::create_with_amp(freq, 1.0, 0.5)
    }
}

impl Modulation for SineModulation {
    fn build(&mut self, config: Configuration) {
        let sf = config.sampling_frequency() as usize;
        let mod_buf_size = config.buf_size() as usize;

        let freq = clamp(self.freq as usize, 1, sf / 2);

        let d = gcd(sf, freq);

        let n = mod_buf_size / d / (mod_buf_size / sf);
        let rep = freq / d;

        self.buffer = Vec::with_capacity(n);

        let offset = self.offset;
        let amp = self.amp;
        for i in 0..n {
            let tamp = ((2 * rep * i) as Float / n as Float) % 2.0;
            let tamp = if tamp > 1. { 2. - tamp } else { tamp };
            let tamp = clamp(offset + (tamp - 0.5) * amp, 0.0, 1.0);
            self.buffer.push((tamp * 255.0) as u8);
        }
    }

    fn buffer(&self) -> &[u8] {
        &self.buffer
    }

    fn sent(&mut self) -> &mut usize {
        &mut self.sent
    }
}
