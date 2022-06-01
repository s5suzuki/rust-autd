/*
 * File: mod.rs
 * Project: ecat_thread
 * Created Date: 03/05/2022
 * Author: Shun Suzuki
 * -----
 * Last Modified: 31/05/2022
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2022 Shun Suzuki. All rights reserved.
 *
 */

mod error_handler;
#[cfg(target_os = "macos")]
mod macos;
mod mode;
#[cfg(all(unix, not(target_os = "macos")))]
mod unix;
mod utils;
#[cfg(windows)]
mod win32;

#[cfg(windows)]
pub use win32::EcatThreadHandler;

#[cfg(all(unix, not(target_os = "macos")))]
pub use unix::EcatThreadHandler;

#[cfg(target_os = "macos")]
pub use macos::EcatThreadHandler;

pub use error_handler::EcatErrorHandler;
pub use mode::*;
