/*
 * File: lib.rs
 * Project: src
 * Created Date: 07/08/2020
 * Author: Shun Suzuki
 * -----
 * Last Modified: 31/12/2020
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2020 Hapis Lab. All rights reserved.
 *
 */

pub mod debug_link;

use std::error::Error;

/// Link is a interface to the AUTD device.
pub trait Link: Send {
    fn open(&mut self) -> Result<(), Box<dyn Error>>;
    fn close(&mut self) -> Result<(), Box<dyn Error>>;
    fn send(&mut self, data: &[u8]) -> Result<(), Box<dyn Error>>;
    fn read(&mut self, data: &mut [u8], buffer_len: usize) -> Result<(), Box<dyn Error>>;
    fn is_open(&self) -> bool;
}
