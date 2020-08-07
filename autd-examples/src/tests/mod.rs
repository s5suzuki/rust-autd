/*
 * File: mod.rs
 * Project: tests
 * Created Date: 16/12/2019
 * Author: Shun Suzuki
 * -----
 * Last Modified: 30/06/2020
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2019 Hapis Lab. All rights reserved.
 *
 */

mod bessel_test;
#[cfg(feature = "csvgain")]
mod csvgain_test;
#[cfg(feature = "groupedgain")]
mod groupedgain_test;
#[cfg(feature = "hologain")]
mod hologain_test;
mod point_sequence;
mod simple_test;
mod soft_stm_test;
mod test_runner;
#[cfg(feature = "wavmodulation")]
mod wavmodulation_test;

pub use bessel_test::*;
#[cfg(feature = "csvgain")]
pub use csvgain_test::*;
#[cfg(feature = "groupedgain")]
pub use groupedgain_test::*;
#[cfg(feature = "hologain")]
pub use hologain_test::*;
pub use point_sequence::*;
pub use simple_test::*;
pub use soft_stm_test::*;
pub use test_runner::run;
#[cfg(feature = "wavmodulation")]
pub use wavmodulation_test::*;
