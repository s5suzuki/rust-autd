/*
 * File: modulation.rs
 * Project: src
 * Created Date: 24/05/2021
 * Author: Shun Suzuki
 * -----
 * Last Modified: 29/05/2021
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2021 Hapis Lab. All rights reserved.
 *
 */

use crate::configuration::Configuration;
use anyhow::Result;

/// Modulation contains the amplitude modulation data.
pub trait Modulation: Send {
    fn build(&mut self, config: Configuration) -> Result<()>;
    fn buffer(&self) -> &[u8];
    fn finished(&self) -> bool {
        self.remaining() == 0
    }
    fn remaining(&self) -> usize {
        self.buffer().len() - self.sent()
    }
    fn head(&self) -> *const u8 {
        unsafe { self.buffer().as_ptr().add(self.sent()) }
    }
    fn sent(&self) -> usize;
    fn send(&mut self, sent: usize);
}
