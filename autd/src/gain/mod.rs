/*
 * File: lib.rs
 * Project: src
 * Created Date: 22/05/2020
 * Author: Shun Suzuki
 * -----
 * Last Modified: 07/08/2020
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2020 Hapis Lab. All rights reserved.
 *
 */

pub mod primitives;

use crate::geometry::Geometry;
use std::f64::consts::PI;

/// Gain contains amplitude and phase of each transducer in the AUTD.
///  Note that the amplitude and phase mean duty ratio and shift duration of Pulse Width Modulation, respectively.
pub trait Gain: Send {
    /// Calculate amplitude and phase of each transducers.
    fn build(&mut self, geometry: &Geometry);
    /// Get amplitude and phase data of all transducers.
    /// The data must be [0-th transducer's phase, 0-th transducer's amplitude, 1-th transducer's phase, ...].
    fn get_data(&self) -> &[u8];
}

/// Adjust amplitude to duty ratio of Pulse Width Modulation.
pub fn adjust_amp(amp: u8) -> u8 {
    let d = (amp as f64 / 255.0).asin() / PI; // duty (0 ~ 0.5)
    (510.0 * d) as u8
}
