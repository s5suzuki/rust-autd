/*
 * File: gain.rs
 * Project: src
 * Created Date: 24/05/2021
 * Author: Shun Suzuki
 * -----
 * Last Modified: 24/11/2021
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2021 Hapis Lab. All rights reserved.
 *
 */

use crate::{geometry::Geometry, hardware_defined::NUM_TRANS_IN_UNIT};
use anyhow::Result;

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Drive {
    pub phase: u8,
    pub duty: u8,
}

pub type GainData = [Drive; NUM_TRANS_IN_UNIT];

/// Gain contains amplitude and phase of each transducer in the AUTD.
/// Note that the amplitude means duty ratio of Pulse Width Modulation, respectively.
pub trait Gain {
    fn build(&mut self, geometry: &Geometry) -> Result<()>;
    fn rebuild(&mut self, geometry: &Geometry) -> Result<()>;
    fn data(&self) -> &[GainData];
    fn take(self) -> Vec<GainData>;
    fn built(&self) -> bool;
}
