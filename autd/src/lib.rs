/*
 * File: autd.rs
 * Project: autd
 * Created Date: 02/09/2019
 * Author: Shun Suzuki
 * -----
 * Last Modified: 30/12/2020
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2019 Hapis Lab. All rights reserved.
 *
 */

#[macro_use]
extern crate bitflags;

#[cfg(feature = "double")]
mod typedef {
    pub type Float = f64;
    pub const PI: Float = std::f64::consts::PI;
}
#[cfg(not(feature = "double"))]
mod typedef {
    pub type Float = f32;
    pub const PI: Float = std::f32::consts::PI;
}

pub use typedef::*;

pub mod controller;
mod core;
pub mod gain;
pub mod geometry;
pub mod link;
pub mod modulation;
pub mod prelude;
pub mod sequence;
pub mod utils;

pub use crate::core::configuration::{Configuration, ModBufSize, ModSamplingFreq};
pub use crate::core::consts;
