/*
 * File: link.rs
 * Project: src
 * Created Date: 24/05/2021
 * Author: Shun Suzuki
 * -----
 * Last Modified: 26/05/2021
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2021 Hapis Lab. All rights reserved.
 *
 */

use anyhow::Result;

/// Link is a interface to the AUTD device.
pub trait Link: Send {
    fn open(&mut self) -> Result<()>;
    fn close(&mut self) -> Result<()>;
    fn send(&mut self, data: &[u8]) -> Result<bool>;
    fn read(&mut self, data: &mut [u8]) -> Result<bool>;
    fn is_open(&self) -> bool;
}
