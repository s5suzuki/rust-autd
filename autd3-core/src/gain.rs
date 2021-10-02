/*
 * File: gain.rs
 * Project: src
 * Created Date: 24/05/2021
 * Author: Shun Suzuki
 * -----
 * Last Modified: 02/10/2021
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2021 Hapis Lab. All rights reserved.
 *
 */

use crate::{geometry::Geometry, hardware_defined::DataArray};
use anyhow::Result;
/// Gain contains amplitude and phase of each transducer in the AUTD.
/// Note that the amplitude means duty ratio of Pulse Width Modulation, respectively.
pub trait Gain {
    fn build(&mut self, geometry: &Geometry) -> Result<()>;
    fn rebuild(&mut self, geometry: &Geometry) -> Result<()>;
    fn data(&self) -> &[DataArray];
    fn take(self) -> Vec<DataArray>;
    fn built(&self) -> bool;
}
