/*
 * File: sequence.rs
 * Project: src
 * Created Date: 24/05/2021
 * Author: Shun Suzuki
 * -----
 * Last Modified: 16/12/2021
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2021 Hapis Lab. All rights reserved.
 *
 */

use crate::{
    error::AutdError,
    gain::Gain,
    geometry::{Geometry, Vector3},
    hardware_defined::{
        self, CPUControlFlags, Drive, FPGAControlFlags, GAIN_SEQ_BUFFER_SIZE_MAX,
        NUM_TRANS_IN_UNIT, POINT_SEQ_BUFFER_SIZE_MAX, SEQ_BASE_FREQ,
    },
    interface::IDatagramBody,
};
use anyhow::Result;
use autd3_traits::Sequence;

pub trait Sequence: IDatagramBody {
    fn set_freq(&mut self, freq: f64) -> f64;
    fn freq(&self) -> f64;
    fn sampling_freq(&self) -> f64;
    fn sampling_freq_div(&mut self) -> &mut usize;
    fn wait_on_sync(&mut self) -> &mut bool;
}

#[repr(C)]
struct SeqFocus {
    buf: [u16; 4],
}

impl SeqFocus {
    pub(crate) fn set(&mut self, x: i32, y: i32, z: i32, duty: u8) {
        self.buf[0] = (x & 0xFFFF) as u16;
        self.buf[1] =
            ((y << 2) & 0xFFFC) as u16 | ((x >> 30) & 0x0002) as u16 | ((x >> 16) & 0x0001) as u16;
        self.buf[2] =
            ((z << 4) & 0xFFF0) as u16 | ((y >> 28) & 0x0008) as u16 | ((y >> 14) & 0x0007) as u16;
        self.buf[3] = (((duty as u16) << 6) & 0x3FC0) as u16
            | ((z >> 26) & 0x0020) as u16
            | ((z >> 12) & 0x001F) as u16;
    }
}

#[derive(Sequence)]
pub struct PointSequence {
    control_points: Vec<(Vector3, u8)>,
    sample_freq_div: usize,
    wait_on_sync: bool,
    sent: usize,
}

impl PointSequence {
    pub fn new() -> Self {
        Self::with_control_points(vec![])
    }

    pub fn with_control_points(control_points: Vec<(Vector3, u8)>) -> Self {
        Self {
            control_points,
            sample_freq_div: 1,
            wait_on_sync: false,
            sent: 0,
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

impl IDatagramBody for PointSequence {
    fn init(&mut self) {
        self.sent = 0;
    }

    fn pack(&mut self, geometry: &Geometry, tx: &mut hardware_defined::TxDatagram) -> Result<()> {
        let header = tx.header_mut();

        if self.wait_on_sync {
            header.cpu_flag |= CPUControlFlags::WAIT_ON_SYNC;
        }
        header.fpga_flag |= FPGAControlFlags::OUTPUT_ENABLE;
        header.fpga_flag |= FPGAControlFlags::SEQ_MODE;
        header.fpga_flag &= !FPGAControlFlags::SEQ_GAIN_MODE;

        if self.is_finished() {
            return Ok(());
        }

        tx.set_num_bodies(geometry.num_devices());
        let header = tx.header_mut();

        let mut offset = 1;
        header.cpu_flag |= CPUControlFlags::WRITE_BODY;
        if self.sent == 0 {
            header.cpu_flag |= CPUControlFlags::SEQ_BEGIN;
            for d in tx.body_data_mut::<[u16; NUM_TRANS_IN_UNIT]>() {
                d[1] = (self.sample_freq_div - 1) as _;
                d[2] = (geometry.wavelength * 1000.0).round() as _;
            }
            offset += 4;
        }
        let send_size = (self.control_points.len() - self.sent).clamp(
            0,
            (NUM_TRANS_IN_UNIT - offset) * std::mem::size_of::<u16>()
                / std::mem::size_of::<SeqFocus>(),
        );
        if self.sent + send_size >= self.control_points.len() {
            let header = tx.header_mut();
            header.cpu_flag |= CPUControlFlags::SEQ_END;
        }
        for d in tx.body_data_mut::<[u16; NUM_TRANS_IN_UNIT]>() {
            d[0] = send_size as _;
        }

        let fixed_num_unit = 256.0 / geometry.wavelength;
        for device in geometry.devices() {
            for (f, c) in tx
                .body_data_mut::<SeqFocus>()
                .iter_mut()
                .zip(self.control_points[self.sent..].iter().take(send_size))
            {
                let v = device.local_position(c.0) * fixed_num_unit;
                f.set(v[0] as _, v[1] as _, v[2] as _, c.1);
            }
        }
        self.sent += send_size;

        Ok(())
    }

    fn is_finished(&self) -> bool {
        self.sent == self.control_points.len()
    }
}

#[repr(u16)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum GainMode {
    DutyPhaseFull = 1,
    PhaseFull = 2,
    PhaseHalf = 4,
}

#[derive(Sequence)]
pub struct GainSequence {
    gains: Vec<Vec<Drive>>,
    sample_freq_div: usize,
    gain_mode: GainMode,
    wait_on_sync: bool,
    sent: usize,
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
            wait_on_sync: false,
            sent: 0,
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

    pub fn gain_mode(&mut self) -> &mut GainMode {
        &mut self.gain_mode
    }
}

#[repr(C)]
struct PhaseDrive {
    pub phase0: u8,
    pub phase1: u8,
}

#[repr(C)]
struct HalfPhaseDrive {
    phase01: u8,
    phase23: u8,
}

impl HalfPhaseDrive {
    pub fn set(&mut self, phase0: u8, phase1: u8, phase2: u8, phase3: u8) {
        self.phase01 = (phase1 & 0xF0) | ((phase0 >> 4) & 0x0F);
        self.phase23 = (phase3 & 0xF0) | ((phase2 >> 4) & 0x0F);
    }
}

impl IDatagramBody for GainSequence {
    fn init(&mut self) {
        self.sent = 0;
    }

    fn pack(&mut self, geometry: &Geometry, tx: &mut hardware_defined::TxDatagram) -> Result<()> {
        let header = tx.header_mut();

        if self.wait_on_sync {
            header.cpu_flag |= CPUControlFlags::WAIT_ON_SYNC;
        }
        header.fpga_flag |= FPGAControlFlags::OUTPUT_ENABLE;
        header.fpga_flag |= FPGAControlFlags::SEQ_MODE;
        header.fpga_flag |= FPGAControlFlags::SEQ_GAIN_MODE;

        if self.is_finished() {
            return Ok(());
        }

        tx.set_num_bodies(geometry.num_devices());

        let header = tx.header_mut();
        header.cpu_flag |= CPUControlFlags::WRITE_BODY;

        let sent = self.gain_mode as usize;
        if self.sent == 0 {
            header.cpu_flag |= CPUControlFlags::SEQ_BEGIN;
            for d in tx.body_data_mut::<[u16; NUM_TRANS_IN_UNIT]>() {
                d[0] = sent as _;
                d[1] = (self.sample_freq_div - 1) as _;
                d[2] = self.gains.len() as _;
            }
            self.sent += 1;
            return Ok(());
        }

        if self.sent + sent > self.gains.len() {
            header.cpu_flag |= CPUControlFlags::SEQ_END;
        }

        let gain_idx = self.sent - 1;
        match self.gain_mode {
            GainMode::DutyPhaseFull => tx
                .body_data_mut::<Drive>()
                .copy_from_slice(&self.gains[gain_idx]),
            GainMode::PhaseFull => {
                let zeros = if gain_idx + 1 > self.gains.len() {
                    vec![
                        Drive {
                            phase: 0x00,
                            duty: 0x00,
                        };
                        self.gains[gain_idx].len()
                    ]
                } else {
                    vec![]
                };
                for ((dst, src0), src1) in tx
                    .body_data_mut::<PhaseDrive>()
                    .iter_mut()
                    .zip(self.gains[gain_idx].iter())
                    .zip(if gain_idx + 1 > self.gains.len() {
                        zeros.iter()
                    } else {
                        self.gains[gain_idx + 1].iter()
                    })
                {
                    dst.phase0 = src0.phase;
                    dst.phase1 = src1.phase;
                }
            }
            GainMode::PhaseHalf => {
                let zeros = if gain_idx + 1 > self.gains.len() {
                    vec![
                        Drive {
                            phase: 0x00,
                            duty: 0x00,
                        };
                        self.gains[gain_idx].len()
                    ]
                } else {
                    vec![]
                };
                for ((((dst, src0), src1), src2), src3) in tx
                    .body_data_mut::<HalfPhaseDrive>()
                    .iter_mut()
                    .zip(self.gains[gain_idx].iter())
                    .zip(if gain_idx + 1 > self.gains.len() {
                        zeros.iter()
                    } else {
                        self.gains[gain_idx + 1].iter()
                    })
                    .zip(if gain_idx + 2 > self.gains.len() {
                        zeros.iter()
                    } else {
                        self.gains[gain_idx + 2].iter()
                    })
                    .zip(if gain_idx + 3 > self.gains.len() {
                        zeros.iter()
                    } else {
                        self.gains[gain_idx + 3].iter()
                    })
                {
                    dst.set(src0.phase, src1.phase, src2.phase, src3.phase);
                }
            }
        }

        self.sent += sent;

        Ok(())
    }

    fn is_finished(&self) -> bool {
        self.sent == self.gains.len() + 1
    }
}

impl Default for GainSequence {
    fn default() -> Self {
        Self::new()
    }
}
