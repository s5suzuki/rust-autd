/*
 * File: modulation.rs
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

use crate::interface::IDatagramHeader;

/// Modulation contains the amplitude modulation data.
pub trait Modulation: IDatagramHeader {
    fn build(&mut self) -> Result<()>;
    fn rebuild(&mut self) -> Result<()>;
    fn buffer(&self) -> &[u8];
    fn sampling_frequency_division(&mut self) -> &mut usize;
    fn sampling_freq(&self) -> f64;
}
