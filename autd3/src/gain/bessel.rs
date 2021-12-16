/*
 * File: bessel.rs
 * Project: gain
 * Created Date: 28/05/2021
 * Author: Shun Suzuki
 * -----
 * Last Modified: 16/12/2021
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2021 Hapis Lab. All rights reserved.
 *
 */

use std::f64::consts::PI;

use anyhow::Result;
use autd3_core::{
    gain::Gain,
    geometry::{Geometry, Matrix4x4, Vector3, Vector4},
    hardware_defined::Drive,
};
use autd3_traits::Gain;

/// Gain to produce Bessel Beam
#[derive(Gain)]
pub struct Bessel {
    data: Vec<Drive>,
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
        let theta_v = v.norm().asin();
        let v = nalgebra::base::Unit::new_normalize(v);
        let rot = Matrix4x4::from_axis_angle(&v, -theta_v);

        let duty = self.duty;
        let wavenum = 2.0 * PI / geometry.wavelength;
        for (trans, data) in geometry.transducers().zip(self.data.iter_mut()) {
            let r = trans.position() - self.pos;
            let r = Vector4::new(r.x, r.y, r.z, 1.0);
            let r = rot * r;
            let dist = self.theta.sin() * (r.x * r.x + r.y * r.y).sqrt() - self.theta.cos() * r.z;
            let phase = wavenum * dist;
            data.duty = duty;
            data.phase = autd3_core::utils::to_phase(phase);
        }
        Ok(())
    }
}
