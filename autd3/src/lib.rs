/*
 * File: lib.rs
 * Project: src
 * Created Date: 26/05/2021
 * Author: Shun Suzuki
 * -----
 * Last Modified: 30/05/2021
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2021 Hapis Lab. All rights reserved.
 *
 */

mod controller;
mod error;
/// primitive gains
pub mod gain;
/// primitive modulations
pub mod modulation;
pub mod prelude;
mod stm_controller;

pub use controller::Controller;
pub use stm_controller::{StmController, StmTimer};
