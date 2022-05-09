/*
 * File: mod.rs
 * Project: fpga
 * Created Date: 02/05/2022
 * Author: Shun Suzuki
 * -----
 * Last Modified: 02/05/2022
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2022 Hapis Lab. All rights reserved.
 *
 */

mod error;
mod fpga_defined;

pub use error::FPGAError;
pub use fpga_defined::*;
