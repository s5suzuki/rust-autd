/*
 * File: logic.rs
 * Project: src
 * Created Date: 24/05/2021
 * Author: Shun Suzuki
 * -----
 * Last Modified: 03/06/2021
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2021 Hapis Lab. All rights reserved.
 *
 */

use std::sync::atomic::{self, AtomicU8};

use crate::{
    configuration::Configuration,
    ec_config::{EC_OUTPUT_FRAME_SIZE, HEADER_SIZE},
    gain::Gain,
    geometry::Geometry,
    hardware_defined::{
        CommandType, DataArray, RxGlobalControlFlags, RxGlobalHeader, SeqFocus, MOD_FRAME_SIZE,
        NUM_TRANS_IN_UNIT,
    },
    modulation::Modulation,
    sequence::PointSequence,
};

static MSG_ID: AtomicU8 = AtomicU8::new(0);

pub struct Logic {}

impl Logic {
    fn get_id() -> u8 {
        MSG_ID.fetch_add(1, atomic::Ordering::SeqCst);
        let _ =
            MSG_ID.compare_exchange(0xFF, 1, atomic::Ordering::SeqCst, atomic::Ordering::SeqCst);
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
        cmd: CommandType,
        flag: RxGlobalControlFlags,
        data: &mut [u8],
        msg_id: &mut u8,
    ) {
        let header = data.as_mut_ptr() as *mut RxGlobalHeader;
        *msg_id = Self::get_id();
        unsafe {
            (*header).msg_id = *msg_id;
            (*header).ctrl_flag = flag;
            (*header).mod_size = 0;
            (*header).command = cmd;
        }
    }

    pub fn pack_header_mod<M: Modulation>(
        modulation: &mut M,
        flag: RxGlobalControlFlags,
        data: &mut [u8],
        msg_id: &mut u8,
    ) {
        Self::pack_header(CommandType::Op, flag, data, msg_id);
        let header = data.as_mut_ptr() as *mut RxGlobalHeader;
        let mod_size = modulation.remaining().clamp(0, MOD_FRAME_SIZE);
        unsafe {
            (*header).mod_size = mod_size as u8;
            if modulation.sent() == 0 {
                (*header).ctrl_flag |= RxGlobalControlFlags::MOD_BEGIN;
            }
            if modulation.sent() + mod_size >= modulation.buffer().len() {
                (*header).ctrl_flag |= RxGlobalControlFlags::MOD_END;
            }
            std::ptr::copy_nonoverlapping(
                modulation.head(),
                (*header).mod_data.as_mut_ptr(),
                mod_size,
            );
        }
        modulation.send(mod_size);
    }

    pub fn pack_body<G: Gain>(gain: &G, data: &mut [u8], size: &mut usize) {
        let num_devices = gain.data().len();

        *size = std::mem::size_of::<RxGlobalHeader>()
            + std::mem::size_of::<u16>() * NUM_TRANS_IN_UNIT * num_devices;
        unsafe {
            let mut cursor = data.as_mut_ptr().add(std::mem::size_of::<RxGlobalHeader>());
            let byte_size = NUM_TRANS_IN_UNIT * std::mem::size_of::<u16>();
            for i in 0..num_devices {
                std::ptr::copy_nonoverlapping(
                    gain.data()[i].as_ptr() as *const u8,
                    cursor,
                    byte_size,
                );
                cursor = cursor.add(byte_size);
            }
        }
    }

    pub fn pack_seq(
        seq: &mut PointSequence,
        geometry: &Geometry,
        data: &mut [u8],
        size: &mut usize,
    ) {
        let num_devices = geometry.num_devices();

        *size = std::mem::size_of::<RxGlobalHeader>()
            + std::mem::size_of::<u16>() * NUM_TRANS_IN_UNIT * num_devices;

        let send_size = seq.remaining().clamp(
            0,
            (EC_OUTPUT_FRAME_SIZE - HEADER_SIZE) / std::mem::size_of::<SeqFocus>(),
        );

        let header = data.as_mut_ptr() as *mut RxGlobalHeader;
        unsafe {
            if seq.sent() == 0 {
                (*header).ctrl_flag |= RxGlobalControlFlags::SEQ_BEGIN;
            }
            if seq.sent() + send_size >= seq.control_points().len() {
                (*header).ctrl_flag |= RxGlobalControlFlags::SEQ_END;
            }
            let mut cursor =
                data.as_mut_ptr().add(std::mem::size_of::<RxGlobalHeader>()) as *mut u16;
            let fixed_num_unit = 255.0 / geometry.wavelength;
            for device in 0..num_devices {
                std::ptr::write(cursor, send_size as u16);
                std::ptr::write(cursor.add(1), seq.sampling_freq_div());
                std::ptr::write(cursor.add(2), (geometry.wavelength * 1000.0) as u16);
                let mut focus_cursor = cursor.add(4) as *mut SeqFocus;
                for i in 0..send_size {
                    let v64 = geometry.local_position(device, seq.control_points()[seq.sent() + i]);
                    let x = v64[0] * fixed_num_unit;
                    let y = v64[1] * fixed_num_unit;
                    let z = v64[2] * fixed_num_unit;
                    (*focus_cursor).set(x as i32, y as i32, z as i32, 0xFF);
                    focus_cursor = focus_cursor.add(1);
                }
                cursor = cursor.add(NUM_TRANS_IN_UNIT);
            }
        }
        seq.send(send_size);
    }

    pub fn pack_sync(config: Configuration, num_devices: usize, data: &mut [u8], size: &mut usize) {
        *size = std::mem::size_of::<RxGlobalHeader>()
            + std::mem::size_of::<u16>() * NUM_TRANS_IN_UNIT * num_devices;
        unsafe {
            let mut cursor =
                data.as_mut_ptr().add(std::mem::size_of::<RxGlobalHeader>()) as *mut u16;
            for _ in 0..num_devices {
                std::ptr::write(cursor, config.mod_buf_size());
                std::ptr::write(cursor.add(1), config.mod_sampling_frequency_division());
                cursor = cursor.add(NUM_TRANS_IN_UNIT);
            }
        }
    }

    pub fn pack_delay(delays: &[DataArray], num_devices: usize, data: &mut [u8], size: &mut usize) {
        *size = std::mem::size_of::<RxGlobalHeader>()
            + std::mem::size_of::<u16>() * NUM_TRANS_IN_UNIT * num_devices;
        unsafe {
            let mut cursor =
                data.as_mut_ptr().add(std::mem::size_of::<RxGlobalHeader>()) as *mut u16;
            for delay in delays {
                std::ptr::copy_nonoverlapping(delay.as_ptr(), cursor, NUM_TRANS_IN_UNIT);
                cursor = cursor.add(NUM_TRANS_IN_UNIT);
            }
        }
    }
}
