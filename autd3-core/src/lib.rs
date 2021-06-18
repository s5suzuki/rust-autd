/*
 * File: lib.rs
 * Project: src
 * Created Date: 24/05/2021
 * Author: Shun Suzuki
 * -----
 * Last Modified: 18/06/2021
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2021 Hapis Lab. All rights reserved.
 *
 */

#[macro_use]
extern crate bitflags;

pub mod ec_config;
pub mod error;
pub mod firmware_version;
pub mod gain;
pub mod geometry;
pub mod hardware_defined;
pub mod link;
pub mod logic;
pub mod modulation;
pub mod sequence;
pub mod utils;
