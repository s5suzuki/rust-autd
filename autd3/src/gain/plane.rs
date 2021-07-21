/*
 * File: plane.rs
 * Project: gain
 * Created Date: 30/05/2021
 * Author: Shun Suzuki
 * -----
 * Last Modified: 21/07/2021
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

/// Gain to create plane wave
#[derive(Gain)]
pub struct Plane {
    data: Vec<DataArray>,
    built: bool,
    duty: u8,
    dir: Vector3,
}

impl Plane {
    /// constructor
    ///
    /// # Arguments
    ///
    /// * `dir` - direction
    ///
    pub fn new(dir: Vector3) -> Self {
        Self::with_duty(dir, 0xFF)
    }

    /// constructor with duty
    ///
    /// # Arguments
    ///
    /// * `dir` - direction
    /// * `duty` - duty ratio
    ///
    pub fn with_duty(dir: Vector3, duty: u8) -> Self {
        Self {
            data: vec![],
            built: false,
            duty,
            dir,
        }
    }

    #[allow(clippy::unnecessary_wraps)]
    fn calc(&mut self, geometry: &Geometry) -> Result<()> {
        let wavelength = geometry.wavelength;
        let duty = self.duty;
        for dev in 0..geometry.num_devices() {
            for i in 0..NUM_TRANS_IN_UNIT {
                let trp = geometry.position_by_local_idx(dev, i);
                let dist = self.dir.dot(&trp);
                let phase = (dist % wavelength) / wavelength;
                let phase = autd3_core::utils::to_phase(phase);
                self.data[dev][i] = autd3_core::utils::pack_to_u16(duty, phase);
            }
        }
        self.built = true;
        Ok(())
    }
}
