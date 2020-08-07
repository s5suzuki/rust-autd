/*
 * File: plane_wave_gain.rs
 * Project: src
 * Created Date: 22/11/2019
 * Author: Shun Suzuki
 * -----
 * Last Modified: 07/08/2020
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2019 Hapis Lab. All rights reserved.
 *
 */

use crate::consts::NUM_TRANS_IN_UNIT;
use crate::geometry::Geometry;
use crate::geometry::Vector3;

use super::super::adjust_amp;
use super::super::Gain;
use crate::consts::ULTRASOUND_WAVELENGTH;

/// Gain to create plane wave.
pub struct PlaneWaveGain {
    dir: Vector3,
    amp: u8,
    data: Option<Vec<u8>>,
}

impl PlaneWaveGain {
    /// Generate PlaneWaveGain.
    ///
    /// # Arguments
    ///
    /// * `dir` - Wave direction.
    ///
    pub fn create(dir: Vector3) -> Box<PlaneWaveGain> {
        PlaneWaveGain::create_with_amp(dir, 0xff)
    }

    /// Generate PlaneWaveGain.
    ///
    /// # Arguments
    ///
    /// * `dir` - Wave direction.
    /// * `amp` - Amplitude of the wave.
    ///
    pub fn create_with_amp(dir: Vector3, amp: u8) -> Box<PlaneWaveGain> {
        Box::new(PlaneWaveGain {
            dir,
            amp,
            data: None,
        })
    }
}

impl Gain for PlaneWaveGain {
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
        let dir = self.dir;
        let amp = self.amp;
        for i in 0..num_trans {
            let trp = geometry.position(i);
            let dist = dir.dot(&trp);
            let phase = (dist % ULTRASOUND_WAVELENGTH) / ULTRASOUND_WAVELENGTH;
            let amp: u8 = amp;
            let phase = (255.0 * (1.0 - phase)) as u8;
            let d = adjust_amp(amp);
            let s = phase;
            data.push(s);
            data.push(d);
        }
        self.data = Some(data);
    }
}
