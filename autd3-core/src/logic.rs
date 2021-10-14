/*
 * File: logic.rs
 * Project: src
 * Created Date: 24/05/2021
 * Author: Shun Suzuki
 * -----
 * Last Modified: 14/10/2021
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2021 Hapis Lab. All rights reserved.
 *
 */

use std::sync::atomic::{self, AtomicU8};

use crate::{
    ec_config::{EC_OUTPUT_FRAME_SIZE, HEADER_SIZE},
    gain::Gain,
    geometry::Geometry,
    hardware_defined::{
        CPUControlFlags, FPGAControlFlags, GlobalHeader, SeqFocus, MOD_FRAME_SIZE, MSG_NORMAL_BASE,
        NUM_TRANS_IN_UNIT,
    },
    modulation::Modulation,
    sequence::{GainSequence, PointSequence, Sequence},
};

static MSG_ID: AtomicU8 = AtomicU8::new(MSG_NORMAL_BASE);

pub struct Logic {}

impl Logic {
    pub fn get_id() -> u8 {
        MSG_ID.fetch_add(1, atomic::Ordering::SeqCst);
        let _ = MSG_ID.compare_exchange(
            0xFF,
            MSG_NORMAL_BASE,
            atomic::Ordering::SeqCst,
            atomic::Ordering::SeqCst,
        );
        MSG_ID.load(atomic::Ordering::SeqCst)
    }

    pub fn is_msg_processed(num_devices: usize, msg_id: u8, rx: &[u8]) -> bool {
        let mut processed = 0;
        for dev in 0..num_devices {
            let proc_id = rx[dev * 2 + 1];
            if msg_id == proc_id {
                processed += 1;
            }
        }
        processed == num_devices
    }

    pub fn pack_header(
        msg_id: u8,
        fpga_flag: FPGAControlFlags,
        cpu_flag: CPUControlFlags,
        data: &mut [u8],
    ) {
        let header = data.as_mut_ptr() as *mut GlobalHeader;
        unsafe {
            (*header).msg_id = msg_id;
            (*header).fpga_flag = fpga_flag;
            (*header).cpu_flag = cpu_flag;
            (*header).mod_size = 0;
        }
    }

    pub fn pack_header_mod<M: Modulation>(
        modulation: &mut M,
        fpga_flag: FPGAControlFlags,
        cpu_flag: CPUControlFlags,
        data: &mut [u8],
    ) -> u8 {
        let msg_id = Self::get_id();
        Self::pack_header(msg_id, fpga_flag, cpu_flag, data);
        if modulation.finished() {
            return msg_id;
        }

        unsafe {
            let header = data.as_mut_ptr() as *mut GlobalHeader;
            let mut offset = 0;
            if modulation.sent() == 0 {
                (*header).cpu_flag |= CPUControlFlags::MOD_BEGIN;
                let div = (*modulation.sampling_frequency_division() - 1) as u16;
                (*header).mod_data[0] = (div & 0xFF) as u8;
                (*header).mod_data[1] = (div >> 8 & 0xFF) as u8;
                offset += 2;
            }
            let mod_size = modulation.remaining().clamp(0, MOD_FRAME_SIZE - offset);
            (*header).mod_size = mod_size as u8;
            if modulation.sent() + mod_size >= modulation.buffer().len() {
                (*header).cpu_flag |= CPUControlFlags::MOD_END;
            }
            std::ptr::copy_nonoverlapping(
                modulation.head(),
                (*header).mod_data.as_mut_ptr().add(offset),
                mod_size,
            );
            modulation.send(mod_size);
        }

        msg_id
    }

    pub fn pack_body<G: Gain>(gain: &G, data: &mut [u8]) -> usize {
        let num_devices = gain.data().len();

        let size = std::mem::size_of::<GlobalHeader>()
            + std::mem::size_of::<u16>() * NUM_TRANS_IN_UNIT * num_devices;
        unsafe {
            let header = data.as_mut_ptr() as *mut GlobalHeader;
            (*header).cpu_flag |= CPUControlFlags::WRITE_BODY;

            let mut cursor = data.as_mut_ptr().add(std::mem::size_of::<GlobalHeader>()) as *mut u16;
            for i in 0..num_devices {
                std::ptr::copy_nonoverlapping(gain.data()[i].as_ptr(), cursor, NUM_TRANS_IN_UNIT);
                cursor = cursor.add(NUM_TRANS_IN_UNIT);
            }
        }
        size
    }

    pub fn pack_seq(seq: &mut PointSequence, geometry: &Geometry, data: &mut [u8]) -> usize {
        if seq.finished() {
            return std::mem::size_of::<GlobalHeader>();
        }

        let num_devices = geometry.num_devices();

        let size = std::mem::size_of::<GlobalHeader>()
            + std::mem::size_of::<u16>() * NUM_TRANS_IN_UNIT * num_devices;

        let header = data.as_mut_ptr() as *mut GlobalHeader;
        let mut offset = 1;
        unsafe {
            (*header).cpu_flag |= CPUControlFlags::WRITE_BODY;

            let mut cursor = data.as_mut_ptr().add(std::mem::size_of::<GlobalHeader>()) as *mut u16;
            if seq.sent() == 0 {
                (*header).cpu_flag |= CPUControlFlags::SEQ_BEGIN;
                for i in 0..num_devices {
                    cursor
                        .add(i * NUM_TRANS_IN_UNIT + 1)
                        .write((*seq.sampling_freq_div() - 1) as u16);
                    cursor
                        .add(i * NUM_TRANS_IN_UNIT + 2)
                        .write((geometry.wavelength * 1000.0) as u16);
                }
                offset += 4;
            }

            let send_size = seq.remaining().clamp(
                0,
                (EC_OUTPUT_FRAME_SIZE - HEADER_SIZE - offset * std::mem::size_of::<u16>())
                    / std::mem::size_of::<SeqFocus>(),
            );

            if seq.sent() + send_size >= seq.size() {
                (*header).cpu_flag |= CPUControlFlags::SEQ_END;
                (*header).fpga_flag |= FPGAControlFlags::OUTPUT_ENABLE;
            }

            let fixed_num_unit = 256.0 / geometry.wavelength;
            for device in 0..num_devices {
                std::ptr::write(cursor, send_size as u16);
                let mut focus_cursor = cursor.add(offset) as *mut SeqFocus;
                for i in 0..send_size {
                    let cp = seq.control_points()[seq.sent() + i];
                    let v64 = geometry.local_position(device, cp.0) * fixed_num_unit;
                    (*focus_cursor).set(v64[0] as i32, v64[1] as i32, v64[2] as i32, cp.1);
                    focus_cursor = focus_cursor.add(1);
                }
                cursor = cursor.add(NUM_TRANS_IN_UNIT);
            }
            seq.send(send_size);
        }

        size
    }

    pub fn pack_gain_seq(seq: &mut GainSequence, geometry: &Geometry, data: &mut [u8]) -> usize {
        if seq.finished() {
            return std::mem::size_of::<GlobalHeader>();
        }

        let num_devices = geometry.num_devices();

        let size = std::mem::size_of::<GlobalHeader>()
            + std::mem::size_of::<u16>() * NUM_TRANS_IN_UNIT * num_devices;

        let header = data.as_mut_ptr() as *mut GlobalHeader;
        let seq_sent = *seq.gain_mode() as usize;
        unsafe {
            (*header).cpu_flag |= CPUControlFlags::WRITE_BODY;

            if seq.sent() == 0 {
                let cursor = data.as_mut_ptr().add(std::mem::size_of::<GlobalHeader>()) as *mut u16;
                (*header).cpu_flag |= CPUControlFlags::SEQ_BEGIN;
                for i in 0..num_devices {
                    cursor.add(i * NUM_TRANS_IN_UNIT).write(seq_sent as _);
                    cursor
                        .add(i * NUM_TRANS_IN_UNIT + 1)
                        .write((*seq.sampling_freq_div() - 1) as u16);
                    cursor.add(i * NUM_TRANS_IN_UNIT + 2).write(seq.size() as _);
                }
                seq.send(1);
                return size;
            }

            if seq.sent() + seq_sent > seq.size() {
                (*header).cpu_flag |= CPUControlFlags::SEQ_END;
                (*header).fpga_flag |= FPGAControlFlags::OUTPUT_ENABLE;
            }

            let mut cursor = data.as_mut_ptr().add(std::mem::size_of::<GlobalHeader>()) as *mut u16;
            let gain_idx = seq.sent() - 1;

            match *seq.gain_mode() {
                crate::hardware_defined::GainMode::DutyPhaseFull => {
                    for device in 0..num_devices {
                        std::ptr::copy_nonoverlapping(
                            seq.gains()[gain_idx][device].as_ptr(),
                            cursor,
                            NUM_TRANS_IN_UNIT,
                        );
                        cursor = cursor.add(NUM_TRANS_IN_UNIT);
                    }
                }
                crate::hardware_defined::GainMode::PhaseFull => {
                    for device in 0..num_devices {
                        for i in 0..NUM_TRANS_IN_UNIT {
                            let low = seq.gains()[gain_idx][device][i] & 0x00FF;
                            let high = if gain_idx + 1 >= seq.size() {
                                0x0000
                            } else {
                                (seq.gains()[gain_idx + 1][device][i] << 8) & 0xFF00
                            };
                            cursor.add(i).write(high | low);
                        }
                        cursor = cursor.add(NUM_TRANS_IN_UNIT);
                    }
                }
                crate::hardware_defined::GainMode::PhaseHalf => {
                    for device in 0..num_devices {
                        for i in 0..NUM_TRANS_IN_UNIT {
                            let phase1 = seq.gains()[gain_idx][device][i] >> 4 & 0x000F;
                            let phase2 = if gain_idx + 1 >= seq.size() {
                                0x0000
                            } else {
                                seq.gains()[gain_idx + 1][device][i] & 0x00F0
                            };
                            let phase3 = if gain_idx + 2 >= seq.size() {
                                0x0000
                            } else {
                                (seq.gains()[gain_idx + 2][device][i]) << 4 & 0x0F00
                            };
                            let phase4 = if gain_idx + 3 >= seq.size() {
                                0x0000
                            } else {
                                (seq.gains()[gain_idx + 3][device][i] << 8) & 0xF000
                            };
                            cursor.add(i).write(phase4 | phase3 | phase2 | phase1);
                        }
                        cursor = cursor.add(NUM_TRANS_IN_UNIT);
                    }
                }
            }

            seq.send(seq_sent);
        }

        size
    }

    pub fn pack_delay_offset(
        delay_offsets: &[[u16; NUM_TRANS_IN_UNIT]],
        num_devices: usize,
        data: &mut [u8],
    ) -> usize {
        let size = std::mem::size_of::<GlobalHeader>()
            + std::mem::size_of::<u16>() * NUM_TRANS_IN_UNIT * num_devices;
        let header = data.as_mut_ptr() as *mut GlobalHeader;
        unsafe {
            (*header).cpu_flag |= CPUControlFlags::WRITE_BODY;
            let mut cursor = data.as_mut_ptr().add(std::mem::size_of::<GlobalHeader>()) as *mut u16;
            for delay in delay_offsets {
                std::ptr::copy_nonoverlapping(delay.as_ptr(), cursor, NUM_TRANS_IN_UNIT);
                cursor = cursor.add(NUM_TRANS_IN_UNIT);
            }
        }

        size
    }
}
