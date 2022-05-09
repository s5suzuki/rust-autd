/*
 * File: hardware.rs
 * Project: src
 * Created Date: 02/05/2022
 * Author: Shun Suzuki
 * -----
 * Last Modified: 02/05/2022
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2022 Hapis Lab. All rights reserved.
 *
 */

pub const NUM_TRANS_IN_UNIT: usize = 249;
pub const NUM_TRANS_X: usize = 18;
pub const NUM_TRANS_Y: usize = 14;
pub const TRANS_SPACING_MM: f64 = 10.16;
pub const DEVICE_WIDTH: f64 = 192.0;
pub const DEVICE_HEIGHT: f64 = 151.4;

pub fn is_missing_transducer<T>(x: T, y: T) -> bool
where
    T: PartialEq<usize>,
{
    y == 1 && (x == 1 || x == 2 || x == 16)
}
