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
        self, Drive, GAIN_SEQ_BUFFER_SIZE_MAX, POINT_SEQ_BUFFER_SIZE_MAX, SEQ_BASE_FREQ,
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

impl IDatagramBody for GainSequence {
    fn init(&mut self) {
        self.sent = 0;
    }

    fn pack(&mut self, geometry: &Geometry, tx: &mut hardware_defined::TxDatagram) {
        if *seq_sent == seq.gains().len() + 1 {
            return std::mem::size_of::<GlobalHeader>();
        }

        let num_devices = geometry.num_devices();

        let size = std::mem::size_of::<GlobalHeader>()
            + std::mem::size_of::<u16>() * NUM_TRANS_IN_UNIT * num_devices;

        let header = data.as_mut_ptr() as *mut GlobalHeader;
        let sent = *seq.gain_mode() as usize;
        unsafe {
            (*header).cpu_flag |= CPUControlFlags::WRITE_BODY;

            if *seq_sent == 0 {
                let cursor = data.as_mut_ptr().add(std::mem::size_of::<GlobalHeader>()) as *mut u16;
                (*header).cpu_flag |= CPUControlFlags::SEQ_BEGIN;
                for i in 0..num_devices {
                    cursor.add(i * NUM_TRANS_IN_UNIT).write(sent as _);
                    cursor
                        .add(i * NUM_TRANS_IN_UNIT + 1)
                        .write((*seq.sampling_freq_div() - 1) as u16);
                    cursor
                        .add(i * NUM_TRANS_IN_UNIT + 2)
                        .write(seq.gains().len() as _);
                }
                *seq_sent += 1;
                return size;
            }

            if *seq_sent + sent > seq.gains().len() {
                (*header).cpu_flag |= CPUControlFlags::SEQ_END;
                (*header).fpga_flag |= FPGAControlFlags::OUTPUT_ENABLE;
            }

            let mut cursor = data.as_mut_ptr().add(std::mem::size_of::<GlobalHeader>()) as *mut u16;
            let gain_idx = *seq_sent - 1;
            match *seq.gain_mode() {
                crate::hardware_defined::GainMode::DutyPhaseFull => {
                    for device in 0..num_devices {
                        std::ptr::copy_nonoverlapping(
                            seq.gains()[gain_idx][device].as_ptr() as _,
                            cursor,
                            NUM_TRANS_IN_UNIT,
                        );
                        cursor = cursor.add(NUM_TRANS_IN_UNIT);
                    }
                }
                crate::hardware_defined::GainMode::PhaseFull => {
                    for device in 0..num_devices {
                        for i in 0..NUM_TRANS_IN_UNIT {
                            let low = seq.gains()[gain_idx][device][i].phase;
                            let high = if gain_idx + 1 >= seq.gains().len() {
                                0x00
                            } else {
                                seq.gains()[gain_idx + 1][device][i].phase
                            };
                            cursor.add(i).write(utils::pack_to_u16(high, low));
                        }
                        cursor = cursor.add(NUM_TRANS_IN_UNIT);
                    }
                }
                crate::hardware_defined::GainMode::PhaseHalf => {
                    for device in 0..num_devices {
                        for i in 0..NUM_TRANS_IN_UNIT {
                            let phase1 = seq.gains()[gain_idx][device][i].phase >> 4 & 0x0F;
                            let phase2 = if gain_idx + 1 >= seq.gains().len() {
                                0x00
                            } else {
                                seq.gains()[gain_idx + 1][device][i].phase & 0xF0
                            };
                            let phase3 = if gain_idx + 2 >= seq.gains().len() {
                                0x00
                            } else {
                                seq.gains()[gain_idx + 2][device][i].phase >> 4 & 0x0F
                            };
                            let phase4 = if gain_idx + 3 >= seq.gains().len() {
                                0x00
                            } else {
                                seq.gains()[gain_idx + 3][device][i].phase & 0xF0
                            };
                            cursor
                                .add(i)
                                .write(utils::pack_to_u16(phase4 | phase3, phase2 | phase1));
                        }
                        cursor = cursor.add(NUM_TRANS_IN_UNIT);
                    }
                }
            }
            *seq_sent += sent;
        }

        size
    }

    fn is_finished(&self) -> bool {
        todo!()
    }
}

impl Default for GainSequence {
    fn default() -> Self {
        Self::new()
    }
}
