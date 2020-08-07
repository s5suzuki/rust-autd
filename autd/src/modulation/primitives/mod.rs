/*
 * File: mod.rs
 * Project: primitives
 * Created Date: 22/05/2020
 * Author: Shun Suzuki
 * -----
 * Last Modified: 07/08/2020
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2020 Hapis Lab. All rights reserved.
 *
 */

mod no_modulation;
mod sine_modulation;

pub use no_modulation::NoModulation;
pub use sine_modulation::SineModulation;
