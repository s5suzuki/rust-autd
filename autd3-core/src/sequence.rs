/*
 * File: sequence.rs
 * Project: src
 * Created Date: 24/05/2021
 * Author: Shun Suzuki
 * -----
 * Last Modified: 28/05/2021
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2021 Hapis Lab. All rights reserved.
 *
 */

use crate::{
    error::AutdError,
    geometry::Vector3,
    hardware_defined::{POINT_SEQ_BASE_FREQ, POINT_SEQ_BUFFER_SIZE_MAX},
};
use anyhow::Result;

pub struct PointSequence {
    control_points: Vec<Vector3>,
    sample_freq_div: u16,
    sent: usize,
}

impl PointSequence {
    pub fn new() -> Self {
        Self {
            control_points: vec![],
            sample_freq_div: 1,
            sent: 0,
        }
    }

    pub fn with_control_points(control_points: Vec<Vector3>) -> Self {
        Self {
            control_points,
            sample_freq_div: 1,
            sent: 0,
        }
    }

    pub fn add_point(&mut self, point: Vector3) -> Result<()> {
        if self.control_points.len() + 1 > POINT_SEQ_BUFFER_SIZE_MAX {
            return Err(AutdError::PointSequenceOutOfBuffer(POINT_SEQ_BUFFER_SIZE_MAX).into());
        }
        self.control_points.push(point);
        Ok(())
    }

    pub fn add_points(&mut self, points: &[Vector3]) -> Result<()> {
        if self.control_points.len() + points.len() > POINT_SEQ_BUFFER_SIZE_MAX {
            return Err(AutdError::PointSequenceOutOfBuffer(POINT_SEQ_BUFFER_SIZE_MAX).into());
        }
        self.control_points.extend_from_slice(points);
        Ok(())
    }

    pub fn set_freq(&mut self, freq: f64) -> f64 {
        let sample_freq = self.control_points.len() as f64 * freq;
        let div = (POINT_SEQ_BASE_FREQ as f64 / sample_freq) as u16;
        self.sample_freq_div = div;
        self.freq()
    }

    pub fn freq(&self) -> f64 {
        self.sampling_freq() / self.control_points.len() as f64
    }

    pub fn sampling_freq(&self) -> f64 {
        POINT_SEQ_BASE_FREQ as f64 / self.sample_freq_div as f64
    }

    pub fn sampling_freq_div(&self) -> u16 {
        self.sample_freq_div
    }

    pub fn sent(&self) -> usize {
        self.sent
    }

    pub fn send(&mut self, sent: usize) {
        self.sent += sent
    }

    pub fn control_points(&self) -> &[Vector3] {
        &self.control_points
    }

    pub fn remaining(&self) -> usize {
        self.control_points.len() - self.sent
    }

    pub fn finished(&self) -> bool {
        self.remaining() == 0
    }
}

impl Default for PointSequence {
    fn default() -> Self {
        Self::new()
    }
}
