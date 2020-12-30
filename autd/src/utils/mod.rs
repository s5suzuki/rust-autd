/*
 * File: mod.rs
 * Project: utils
 * Created Date: 30/12/2020
 * Author: Shun Suzuki
 * -----
 * Last Modified: 30/12/2020
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2020 Hapis Lab. All rights reserved.
 *
 */

mod attenuation;
mod directivity;

pub use attenuation::attenuation_coef;
pub use directivity::directivity_t4010a1;
