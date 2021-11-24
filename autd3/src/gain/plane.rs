/*
 * File: plane.rs
 * Project: gain
 * Created Date: 30/05/2021
 * Author: Shun Suzuki
 * -----
 * Last Modified: 24/11/2021
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2021 Hapis Lab. All rights reserved.
 *
 */

use std::f64::consts::PI;

use anyhow::Result;
use autd3_core::{
    gain::{Gain, GainData},
    geometry::{Geometry, Vector3},
};
use autd3_traits::Gain;

/// Gain to create plane wave
#[derive(Gain)]
pub struct Plane {
    data: Vec<GainData>,
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
        let wavenum = 2.0 * PI / geometry.wavelength;
        let duty = self.duty;
        for (dev, data) in geometry.devices().zip(self.data.iter_mut()) {
            for (trans, d) in dev.transducers().zip(data.iter_mut()) {
                let dist = self.dir.dot(trans);
                d.duty = duty;
                d.phase = autd3_core::utils::to_phase(wavenum * dist);
            }
        }
        Ok(())
    }
}
