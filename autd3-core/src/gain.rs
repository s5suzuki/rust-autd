/*
 * File: gain.rs
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

use crate::{geometry::Geometry, hardware_defined::Drive, interface::IDatagramBody};
use anyhow::Result;

/// Gain contains amplitude and phase of each transducer in the AUTD.
/// Note that the amplitude means duty ratio of Pulse Width Modulation, respectively.
pub trait Gain: IDatagramBody {
    fn build(&mut self, geometry: &Geometry) -> Result<()>;
    fn rebuild(&mut self, geometry: &Geometry) -> Result<()>;
    fn data(&self) -> &[Drive];
    fn take(self) -> Vec<Drive>;
    fn built(&self) -> bool;
}
