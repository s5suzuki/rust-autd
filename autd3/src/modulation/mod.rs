/*
 * File: mod.rs
 * Project: modulation
 * Created Date: 27/05/2021
 * Author: Shun Suzuki
 * -----
 * Last Modified: 27/05/2021
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2021 Hapis Lab. All rights reserved.
 *
 */

mod sine;
mod r#static;

pub use r#static::Static;
pub use sine::Sine;
