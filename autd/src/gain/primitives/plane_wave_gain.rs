/*
 * File: plane_wave_gain.rs
 * Project: src
 * Created Date: 22/11/2019
 * Author: Shun Suzuki
 * -----
 * Last Modified: 31/12/2020
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2019 Hapis Lab. All rights reserved.
 *
 */

use crate::geometry::Vector3;
use crate::{consts::DataArray, geometry::Geometry};
use crate::{consts::NUM_TRANS_IN_UNIT, Float};

use super::super::adjust_amp;
use super::super::Gain;

/// Gain to create plane wave.
pub struct PlaneWaveGain {
    dir: Vector3,
    duty: u8,
    data: Option<Vec<DataArray>>,
}

impl PlaneWaveGain {
    /// Generate PlaneWaveGain.
    ///
    /// # Arguments
    ///
    /// * `dir` - Wave direction.
    ///
    pub fn create(dir: Vector3) -> Self {
        Self::create_with_duty(dir, 0xff)
    }

    /// Generate PlaneWaveGain.
    ///
    /// # Arguments
    ///
    /// * `dir` - Wave direction.
    /// * `amp` - Amplitude of the wave.
    ///
    pub fn create_with_amp(dir: Vector3, amp: Float) -> Self {
        Self::create_with_duty(dir, adjust_amp(amp))
    }

    /// Generate PlaneWaveGain.
    ///
    /// # Arguments
    ///
    /// * `dir` - Wave direction.
    /// * `duty` - Duty ratio of input signal to transducer.
    ///
    pub fn create_with_duty(dir: Vector3, duty: u8) -> Self {
        PlaneWaveGain {
            dir,
            duty,
            data: None,
        }
    }
}

impl Gain for PlaneWaveGain {
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

        let dir = self.dir;
        let duty = (self.duty as u16) << 8;
        let wavelength = geometry.wavelength();
        for dev in 0..num_devices {
            let mut buf: DataArray = unsafe { std::mem::zeroed() };
            for (i, b) in buf.iter_mut().enumerate().take(NUM_TRANS_IN_UNIT) {
                let trp = geometry.position_by_local_idx(dev, i);
                let dist = dir.dot(&trp);
                let phase = (dist % wavelength) / wavelength;
                let phase = (255.0 * (1.0 - phase)) as u16;
                *b = duty | phase;
            }
            data.push(buf);
        }

        self.data = Some(data);
    }
}
