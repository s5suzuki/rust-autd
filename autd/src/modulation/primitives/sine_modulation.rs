/*
 * File: sine_modulation.rs
 * Project: modulation
 * Created Date: 16/11/2019
 * Author: Shun Suzuki
 * -----
 * Last Modified: 07/08/2020
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2019 Hapis Lab. All rights reserved.
 *
 */

use super::super::Modulation;
use crate::consts::MOD_BUF_SIZE;
use crate::consts::MOD_SAMPLING_FREQUENCY;

use num::clamp;
use num::integer::gcd;

/// Sine wave modulation
pub struct SineModulation {}

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
    pub fn create_with_amp(freq: i32, amp: f64, offset: f64) -> Modulation {
        let sf = MOD_SAMPLING_FREQUENCY as i32;
        let freq = clamp(freq, 1, sf / 2);
        let d = gcd(sf, freq);
        let n = MOD_BUF_SIZE as i32 / d;
        let rep = freq / d;
        let mut buffer = Vec::with_capacity(n as usize);

        for i in 0..n {
            let tamp = ((2 * rep * i) as f64 / n as f64) % 2.0;
            let tamp = if tamp > 1.0 { 2.0 - tamp } else { tamp };
            let tamp = clamp(offset + (tamp - 0.5) * amp, 0.0, 1.0);
            buffer.push((tamp * 255.0) as u8);
        }

        Modulation { buffer, sent: 0 }
    }

    /// Generate SineModulation.
    ///
    /// # Arguments
    ///
    /// * `freq` - Frequency of the sine wave.
    ///
    pub fn create(freq: i32) -> Modulation {
        SineModulation::create_with_amp(freq, 1.0, 0.5)
    }
}
