/*
 * File: bessel.rs
 * Project: gain
 * Created Date: 28/05/2021
 * Author: Shun Suzuki
 * -----
 * Last Modified: 19/06/2021
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

/// Gain to produce Bessel Beam
#[derive(Gain)]
pub struct Bessel {
    data: Vec<DataArray>,
    built: bool,
    duty: u8,
    pos: Vector3,
    dir: Vector3,
    theta: f64,
}

impl Bessel {
    /// constructor
    ///
    /// # Arguments
    ///
    /// * `point` - Start point of the beam
    /// * `dir` - Direction of the beam
    /// * `theta` - Angle between the conical wavefront of the beam and the direction
    ///
    pub fn new(pos: Vector3, dir: Vector3, theta: f64) -> Self {
        Self::with_duty(pos, dir, theta, 0xFF)
    }

    /// constructor with duty ratio
    ///
    /// # Arguments
    ///
    /// * `point` - Start point of the beam
    /// * `dir` - Direction of the beam
    /// * `theta` - Angle between the conical wavefront of the beam and the direction
    /// * `duty` - Duty ratio
    ///
    pub fn with_duty(pos: Vector3, dir: Vector3, theta: f64, duty: u8) -> Self {
        Self {
            data: vec![],
            built: false,
            duty,
            pos,
            dir,
            theta,
        }
    }

    #[allow(clippy::unnecessary_wraps)]
    fn calc(&mut self, geometry: &Geometry) -> Result<()> {
        let dir = self.dir.normalize();
        let v = Vector3::new(dir.y, -dir.x, 0.);
        let theta_w = v.norm().asin();

        let duty = (self.duty as u16) << 8;
        let wavelength = geometry.wavelength;
        for dev in 0..geometry.num_devices() {
            for i in 0..NUM_TRANS_IN_UNIT {
                let trp = geometry.position_by_local_idx(dev, i);
                let r = trp - self.pos;
                let xr = r.cross(&v);
                let r =
                    r * theta_w.cos() + xr * theta_w.sin() + v * (v.dot(&r) * (1. - theta_w.cos()));
                let dist =
                    self.theta.sin() * (r.x * r.x + r.y * r.y).sqrt() - self.theta.cos() * r.z;
                let phase = (dist % wavelength) / wavelength;
                let phase = ((256.0 * (1.0 - phase)).round() as u16) & 0x00FF;
                self.data[dev][i] = duty | phase;
            }
        }
        self.built = true;
        Ok(())
    }
}
