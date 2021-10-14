/*
 * File: error.rs
 * Project: src
 * Created Date: 26/05/2021
 * Author: Shun Suzuki
 * -----
 * Last Modified: 14/10/2021
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2021 Hapis Lab. All rights reserved.
 *
 */

use thiserror::Error;

#[derive(Error, Debug)]
pub enum AutdError {
    #[error("Link is closed.")]
    LinkClosed,
    #[error("The maximum frequency division ratio is {0}")]
    FrequencyDivisionRatioOutOfRange(usize),
    #[error("The maximum size of Modulation is {0}")]
    ModulationOutOfBuffer(usize),
    #[error("The maximum size of PointSequence is {0}")]
    PointSequenceOutOfBuffer(usize),
}
