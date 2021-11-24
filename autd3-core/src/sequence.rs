/*
 * File: sequence.rs
 * Project: src
 * Created Date: 24/05/2021
 * Author: Shun Suzuki
 * -----
 * Last Modified: 24/11/2021
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2021 Hapis Lab. All rights reserved.
 *
 */

use crate::{
    error::AutdError,
    gain::{Gain, GainData},
    geometry::{Geometry, Vector3},
    hardware_defined::{
        self, GainMode, GAIN_SEQ_BUFFER_SIZE_MAX, POINT_SEQ_BUFFER_SIZE_MAX, SEQ_BASE_FREQ,
    },
};
use anyhow::Result;
use autd3_traits::Sequence;

pub trait Sequence {
    fn set_freq(&mut self, freq: f64) -> f64;
    fn freq(&self) -> f64;
    fn sampling_freq(&self) -> f64;
    fn sampling_freq_div(&mut self) -> &mut usize;
}

#[derive(Sequence)]
pub struct PointSequence {
    control_points: Vec<(Vector3, u8)>,
    sample_freq_div: usize,
}

impl PointSequence {
    pub fn new() -> Self {
        Self {
            control_points: vec![],
            sample_freq_div: 1,
        }
    }

    pub fn with_control_points(control_points: Vec<(Vector3, u8)>) -> Self {
        Self {
            control_points,
            sample_freq_div: 1,
        }
    }

    pub fn add_point(&mut self, point: Vector3, duty: u8) -> Result<()> {
        if self.control_points.len() + 1 > POINT_SEQ_BUFFER_SIZE_MAX {
            return Err(AutdError::PointSequenceOutOfBuffer(POINT_SEQ_BUFFER_SIZE_MAX).into());
        }
        self.control_points.push((point, duty));
        Ok(())
    }

    pub fn add_points(&mut self, points: &[(Vector3, u8)]) -> Result<()> {
        if self.control_points.len() + points.len() > POINT_SEQ_BUFFER_SIZE_MAX {
            return Err(AutdError::PointSequenceOutOfBuffer(POINT_SEQ_BUFFER_SIZE_MAX).into());
        }
        self.control_points.extend_from_slice(points);
        Ok(())
    }

    pub fn size(&self) -> usize {
        self.control_points.len()
    }

    pub fn control_points(&self) -> &[(Vector3, u8)] {
        &self.control_points
    }
}

impl Default for PointSequence {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Sequence)]
pub struct GainSequence {
    gains: Vec<Vec<GainData>>,
    sample_freq_div: usize,
    gain_mode: GainMode,
}

impl GainSequence {
    pub fn new() -> Self {
        Self::with_gain_mode(GainMode::DutyPhaseFull)
    }

    pub fn with_gain_mode(gain_mode: GainMode) -> Self {
        Self {
            gains: vec![],
            sample_freq_div: 1,
            gain_mode,
        }
    }

    pub fn add_gain<G: Gain>(&mut self, mut gain: G, geometry: &Geometry) -> Result<()> {
        if self.gains.len() + 1 > GAIN_SEQ_BUFFER_SIZE_MAX {
            return Err(AutdError::PointSequenceOutOfBuffer(POINT_SEQ_BUFFER_SIZE_MAX).into());
        }
        gain.build(geometry)?;
        self.gains.push(gain.take());
        Ok(())
    }

    pub fn size(&self) -> usize {
        self.gains.len()
    }

    pub fn gains(&self) -> &[Vec<GainData>] {
        &self.gains
    }

    pub fn gain_mode(&mut self) -> &mut GainMode {
        &mut self.gain_mode
    }
}

impl Default for GainSequence {
    fn default() -> Self {
        Self::new()
    }
}
