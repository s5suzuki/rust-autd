/*
 * File: bessel_beam_gain.rs
 * Project: src
 * Created Date: 22/11/2019
 * Author: Shun Suzuki
 * -----
 * Last Modified: 31/12/2020
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2019 Hapis Lab. All rights reserved.
 * -----
 * The following algorithm is originally developed by Keisuke Hasegawa et al.
 * K. Hasegawa, et al. "Electronically Steerable Ultrasound-Driven Long Narrow Air Stream," Applied Physics Letters, 111, 064104 (2017).
 *
 */

use crate::{
    consts::{DataArray, NUM_TRANS_IN_UNIT},
    geometry::{Geometry, Vector3},
    Float,
};

use super::super::{adjust_amp, Gain};

/// Gain to produce Bessel Beam
pub struct BesselBeamGain {
    point: Vector3,
    dir: Vector3,
    theta_z: Float,
    duty: u8,
    data: Option<Vec<DataArray>>,
}

impl BesselBeamGain {
    /// Generate BesselBeamGain.
    ///
    /// # Arguments
    ///
    /// * `point` - Start point of the beam.
    /// * `dir` - Direction of the beam.
    /// * `theta_z` - Angle between the conical wavefront of the beam and the direction.
    ///
    pub fn create(point: Vector3, dir: Vector3, theta_z: Float) -> BesselBeamGain {
        Self::create_with_duty(point, dir, theta_z, 0xff)
    }

    /// Generate BesselBeamGain.
    ///
    /// # Arguments
    ///
    /// * `point` - Start point of the beam.
    /// * `dir` - Direction of the beam.
    /// * `theta_z` - Angle between the conical wavefront of the beam and the direction.
    /// * `duty` - Duty ratio of input signal to transducer.
    ///
    pub fn create_with_duty(point: Vector3, dir: Vector3, theta_z: Float, duty: u8) -> Self {
        Self {
            point,
            dir,
            theta_z,
            duty,
            data: None,
        }
    }

    /// Generate BesselBeamGain.
    ///
    /// # Arguments
    ///
    /// * `point` - Start point of the beam.
    /// * `dir` - Direction of the beam.
    /// * `theta_z` - Angle between the conical wavefront of the beam and the direction.
    /// * `amp` - Amplitude of the beam (0-1).
    ///
    pub fn create_with_amp(point: Vector3, dir: Vector3, theta_z: Float, amp: Float) -> Self {
        Self::create_with_duty(point, dir, theta_z, adjust_amp(amp))
    }
}

impl Gain for BesselBeamGain {
    fn get_data(&self) -> &[DataArray] {
        assert!(self.data.is_some());
        match &self.data {
            Some(data) => data,
            None => panic!(),
        }
    }

    fn build(&mut self, geometry: &Geometry) {
        if self.data.is_some() {
            return;
        }

        let num_devices = geometry.num_devices();
        let mut data = Vec::with_capacity(num_devices);

        let dir = self.dir.normalize();
        let v = Vector3::new(dir.y, -dir.x, 0.);
        let theta_w = v.norm().asin();
        let point = self.point;
        let theta_z = self.theta_z;

        let duty = (self.duty as u16) << 8;
        let wavelength = geometry.wavelength();
        for dev in 0..num_devices {
            let mut buf: DataArray = unsafe { std::mem::zeroed() };
            for (i, b) in buf.iter_mut().enumerate().take(NUM_TRANS_IN_UNIT) {
                let trp = geometry.position_by_local_idx(dev, i);
                let r = trp - point;
                let xr = r.cross(&v);
                let r =
                    r * theta_w.cos() + xr * theta_w.sin() + v * (v.dot(&r) * (1. - theta_w.cos()));
                let dist = theta_z.sin() * (r.x * r.x + r.y * r.y).sqrt() - theta_z.cos() * r.z;
                let phase = (dist % wavelength) / wavelength;
                let phase = (255.0 * (1.0 - phase)) as u16;
                *b = duty | phase;
            }
            data.push(buf);
        }

        self.data = Some(data);
    }
}
