/*
 * File: error.rs
 * Project: src
 * Created Date: 30/05/2021
 * Author: Shun Suzuki
 * -----
 * Last Modified: 30/05/2021
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2021 Hapis Lab. All rights reserved.
 *
 */

use thiserror::Error;

#[derive(Error, Debug)]
pub enum AutdError {
    #[error("You specified device id ({0}) in Grouped, but only {1} AUTDs are connected.")]
    GroupedOutOfRange(usize, usize),
    #[error("You specified {0} devices in delays, but only {1} devices are connected.")]
    DelayOutOfRange(usize, usize),
}
