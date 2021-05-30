/*
 * File: soem_error.rs
 * Project: src
 * Created Date: 21/11/2019
 * Author: Shun Suzuki
 * -----
 * Last Modified: 27/05/2021
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2019 Hapis Lab. All rights reserved.
 *
 */

use thiserror::Error;

#[derive(Error, Debug)]
pub enum SoemError {
    #[error("No socket connection on {0}")]
    NoSocketConnection(String),
    #[error("The number of slaves you specified: {1}, but found: {0}")]
    SlaveNotFound(u16, u16),
    #[error("One ore more slaves are not responding")]
    NotResponding,
}
