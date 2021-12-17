/*
 * File: link.rs
 * Project: src
 * Created Date: 24/05/2021
 * Author: Shun Suzuki
 * -----
 * Last Modified: 16/12/2021
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2021 Hapis Lab. All rights reserved.
 *
 */

use anyhow::Result;

use crate::hardware_defined::{RxDatagram, TxDatagram};

/// Link is a interface to the AUTD device.
pub trait Link: Send {
    fn open(&mut self) -> Result<()>;
    fn close(&mut self) -> Result<()>;
    fn send(&mut self, tx: &TxDatagram) -> Result<bool>;
    fn receive(&mut self, rx: &mut RxDatagram) -> Result<bool>;
    fn is_open(&self) -> bool;
}
