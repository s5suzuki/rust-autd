/*
 * File: focal_point_gain.rs
 * Project: src
 * Created Date: 15/11/2019
 * Author: Shun Suzuki
 * -----
 * Last Modified: 31/12/2020
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2019 Hapis Lab. All rights reserved.
 *
 */

use crate::{
    consts::{DataArray, NUM_TRANS_IN_UNIT},
    geometry::{Geometry, Vector3},
    Float,
};

use super::super::{adjust_amp, Gain};

/// Gain to produce single focal point
pub struct FocalPointGain {
    point: Vector3,
    duty: u8,
    data: Option<Vec<DataArray>>,
}

impl FocalPointGain {
    /// Generate FocalPointGain.
    ///
    /// # Arguments
    ///
    /// * `point` - focal point.
    ///
    pub fn create(point: Vector3) -> Self {
        Self::create_with_duty(point, 0xff)
    }

    /// Generate FocalPointGain.
    ///
    /// # Arguments
    ///
    /// * `point` - focal point.
    /// * `amp` - amplitude of the focus.
    ///
    pub fn create_with_amp(point: Vector3, amp: Float) -> Self {
        Self::create_with_duty(point, adjust_amp(amp))
    }

    /// Generate FocalPointGain.
    ///
    /// # Arguments
    ///
    /// * `point` - focal point.
    /// * `duty` - Duty ratio of input signal to transducer.
    ///
    pub fn create_with_duty(point: Vector3, duty: u8) -> Self {
        FocalPointGain {
            point,
            duty,
            data: None,
        }
    }
}

impl Gain for FocalPointGain {
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

        let point = self.point;
        let duty = (self.duty as u16) << 8;
        let wavelength = geometry.wavelength();
        for dev in 0..num_devices {
            let mut buf: DataArray = unsafe { std::mem::zeroed() };
            for (i, b) in buf.iter_mut().enumerate().take(NUM_TRANS_IN_UNIT) {
                let trp = geometry.position_by_local_idx(dev, i);
                let dist = (trp - point).norm();
                let phase = (dist % wavelength) / wavelength;
                let phase = (255.0 * (1.0 - phase)) as u16;
                *b = duty | phase;
            }
            data.push(buf);
        }

        self.data = Some(data);
    }
}
