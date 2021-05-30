/*
 * File: focus.rs
 * Project: gain
 * Created Date: 27/05/2021
 * Author: Shun Suzuki
 * -----
 * Last Modified: 30/05/2021
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2021 Hapis Lab. All rights reserved.
 *
 */

use anyhow::Result;
use autd3_core::{
    gain::Gain,
    geometry::{Geometry, Vector3},
    hardware_defined::{DataArray, NUM_TRANS_IN_UNIT},
};
use autd3_traits::Gain;

/// Gain to produce single focal point
#[derive(Gain)]
pub struct Focus {
    data: Vec<DataArray>,
    built: bool,
    duty: u8,
    pos: Vector3,
}

impl Focus {
    /// constructor
    ///
    /// # Arguments
    ///
    /// * `pos` - position of focal point
    ///
    pub fn new(pos: Vector3) -> Self {
        Self::with_duty(pos, 0xFF)
    }

    /// constructor with duty
    ///
    /// # Arguments
    ///
    /// * `pos` - position of focal point
    /// * `duty` - duty ratio
    ///
    pub fn with_duty(pos: Vector3, duty: u8) -> Self {
        Self {
            data: vec![],
            built: false,
            duty,
            pos,
        }
    }

    #[allow(clippy::unnecessary_wraps)]
    fn calc(&mut self, geometry: &Geometry) -> Result<()> {
        let wavelength = geometry.wavelength;
        let duty = ((self.duty as u16) << 8) & 0xFF00;
        for dev in 0..geometry.num_devices() {
            for i in 0..NUM_TRANS_IN_UNIT {
                let trp = geometry.position_by_local_idx(dev, i);
                let dist = (trp - self.pos).norm();
                let f_phase = (dist % wavelength) / wavelength;
                let phase = (255.0 * (1.0 - f_phase)).round() as u16;
                self.data[dev][i] = duty | phase;
            }
        }
        self.built = true;
        Ok(())
    }
}