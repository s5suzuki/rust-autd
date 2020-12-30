/*
 * File: bessel_beam_gain.rs
 * Project: src
 * Created Date: 22/11/2019
 * Author: Shun Suzuki
 * -----
 * Last Modified: 30/12/2020
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2019 Hapis Lab. All rights reserved.
 * -----
 * The following algorithm is originally developed by Keisuke Hasegawa et al.
 * K. Hasegawa, et al. "Electronically Steerable Ultrasound-Driven Long Narrow Air Stream," Applied Physics Letters, 111, 064104 (2017).
 *
 */

use crate::consts::NUM_TRANS_IN_UNIT;
use crate::geometry::Geometry;
use crate::geometry::Vector3;
use crate::Float;

use super::super::adjust_amp;
use super::super::Gain;
use crate::consts::ULTRASOUND_WAVELENGTH;

/// Gain to produce Bessel Beam
pub struct BesselBeamGain {
    point: Vector3,
    dir: Vector3,
    theta_z: Float,
    amp: u8,
    data: Option<Vec<u8>>,
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
    pub fn create(point: Vector3, dir: Vector3, theta_z: Float) -> Box<BesselBeamGain> {
        BesselBeamGain::create_with_amp(point, dir, theta_z, 0xff)
    }

    /// Generate BesselBeamGain.
    ///
    /// # Arguments
    ///
    /// * `point` - Start point of the beam.
    /// * `dir` - Direction of the beam.
    /// * `theta_z` - Angle between the conical wavefront of the beam and the direction.
    /// * `amp` - Amplitude of the beam.
    ///
    pub fn create_with_amp(
        point: Vector3,
        dir: Vector3,
        theta_z: Float,
        amp: u8,
    ) -> Box<BesselBeamGain> {
        Box::new(BesselBeamGain {
            point,
            dir,
            theta_z,
            amp,
            data: None,
        })
    }
}

impl Gain for BesselBeamGain {
    fn get_data(&self) -> &[u8] {
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
        let num_trans = NUM_TRANS_IN_UNIT * num_devices;
        let mut data = Vec::with_capacity(num_trans * 2);

        let dir = self.dir.normalize();
        let v = Vector3::new(dir.y, -dir.x, 0.);
        let theta_w = v.norm().asin();
        let point = self.point;
        let theta_z = self.theta_z;
        for i in 0..num_trans {
            let trp = geometry.position(i);
            let r = trp - point;
            let xr = r.cross(&v);
            let r = r * theta_w.cos() + xr * theta_w.sin() + v * (v.dot(&r) * (1. - theta_w.cos()));
            let dist = theta_z.sin() * (r.x * r.x + r.y * r.y).sqrt() - theta_z.cos() * r.z;
            let phase = (dist % ULTRASOUND_WAVELENGTH) / ULTRASOUND_WAVELENGTH;
            let phase = (255.0 * (1.0 - phase)) as u8;
            let amp: u8 = self.amp;
            let d = adjust_amp(amp);
            let s = phase;
            data.push(s);
            data.push(d);
        }
        self.data = Some(data);
    }
}
