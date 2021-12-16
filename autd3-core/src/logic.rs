/*
 * File: logic.rs
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

// use std::sync::atomic::{self, AtomicU8};

// use crate::{
//     ec_config::{EC_OUTPUT_FRAME_SIZE, HEADER_SIZE},
//     gain::Gain,
//     geometry::Geometry,
//     hardware_defined::{
//         CPUControlFlags, DelayOffset, FPGAControlFlags, GlobalHeader, SeqFocus, MOD_FRAME_SIZE,
//         MSG_NORMAL_BASE, NUM_TRANS_IN_UNIT,
//     },
//     modulation::Modulation,
//     sequence::{GainSequence, PointSequence, Sequence},
//     utils,
// };

// static MSG_ID: AtomicU8 = AtomicU8::new(MSG_NORMAL_BASE);

// pub struct Logic {}

// impl Logic {
//     pub fn get_id() -> u8 {
//         MSG_ID.fetch_add(1, atomic::Ordering::SeqCst);
//         let _ = MSG_ID.compare_exchange(
//             0xFF,
//             MSG_NORMAL_BASE,
//             atomic::Ordering::SeqCst,
//             atomic::Ordering::SeqCst,
//         );
//         MSG_ID.load(atomic::Ordering::SeqCst)
//     }

//     pub fn pack_header(
//         msg_id: u8,
//         fpga_flag: FPGAControlFlags,
//         cpu_flag: CPUControlFlags,
//         data: &mut [u8],
//     ) {
//         let header = data.as_mut_ptr() as *mut GlobalHeader;
//         unsafe {
//             (*header).msg_id = msg_id;
//             (*header).fpga_flag = fpga_flag;
//             (*header).cpu_flag = cpu_flag;
//             (*header).mod_size = 0;
//         }
//     }

//     pub fn pack_header_mod<M: Modulation>(
//         modulation: &mut M,
//         fpga_flag: FPGAControlFlags,
//         cpu_flag: CPUControlFlags,
//         data: &mut [u8],
//         mod_sent: &mut usize,
//     ) -> u8 {
//         let msg_id = Self::get_id();
//         Self::pack_header(msg_id, fpga_flag, cpu_flag, data);
//         if *mod_sent >= modulation.buffer().len() {
//             return msg_id;
//         }

//         unsafe {
//             let header = data.as_mut_ptr() as *mut GlobalHeader;
//             let mut offset = 0;
//             if *mod_sent == 0 {
//                 (*header).cpu_flag |= CPUControlFlags::MOD_BEGIN;
//                 let div = (*modulation.sampling_frequency_division() - 1) as u16;
//                 (*header).mod_data[0] = (div & 0xFF) as u8;
//                 (*header).mod_data[1] = (div >> 8 & 0xFF) as u8;
//                 offset += 2;
//             }
//             let mod_size =
//                 (modulation.buffer().len() - *mod_sent).clamp(0, MOD_FRAME_SIZE - offset);
//             (*header).mod_size = mod_size as u8;
//             if *mod_sent + mod_size >= modulation.buffer().len() {
//                 (*header).cpu_flag |= CPUControlFlags::MOD_END;
//             }
//             std::ptr::copy_nonoverlapping(
//                 modulation.buffer().as_ptr().add(*mod_sent),
//                 (*header).mod_data.as_mut_ptr().add(offset),
//                 mod_size,
//             );
//             *mod_sent += mod_size;
//         }

//         msg_id
//     }

//     pub fn pack_body<G: Gain>(gain: &G, data: &mut [u8]) -> usize {
//         let num_devices = gain.data().len();

//         let size = std::mem::size_of::<GlobalHeader>()
//             + std::mem::size_of::<u16>() * NUM_TRANS_IN_UNIT * num_devices;
//         unsafe {
//             let header = data.as_mut_ptr() as *mut GlobalHeader;
//             (*header).cpu_flag |= CPUControlFlags::WRITE_BODY;

//             let mut cursor = data.as_mut_ptr().add(std::mem::size_of::<GlobalHeader>()) as *mut u16;
//             for i in 0..num_devices {
//                 std::ptr::copy_nonoverlapping(
//                     gain.data()[i].as_ptr() as _,
//                     cursor,
//                     NUM_TRANS_IN_UNIT,
//                 );
//                 cursor = cursor.add(NUM_TRANS_IN_UNIT);
//             }
//         }
//         size
//     }

//     pub fn pack_seq(
//         seq: &mut PointSequence,
//         geometry: &Geometry,
//         data: &mut [u8],
//         seq_sent: &mut usize,
//     ) -> usize {
//         if *seq_sent == seq.control_points().len() {
//             return std::mem::size_of::<GlobalHeader>();
//         }

//         let num_devices = geometry.num_devices();

//         let size = std::mem::size_of::<GlobalHeader>()
//             + std::mem::size_of::<u16>() * NUM_TRANS_IN_UNIT * num_devices;

//         let header = data.as_mut_ptr() as *mut GlobalHeader;
//         let mut offset = 1;
//         unsafe {
//             (*header).cpu_flag |= CPUControlFlags::WRITE_BODY;

//             let mut cursor = data.as_mut_ptr().add(std::mem::size_of::<GlobalHeader>()) as *mut u16;
//             if *seq_sent == 0 {
//                 (*header).cpu_flag |= CPUControlFlags::SEQ_BEGIN;
//                 for i in 0..num_devices {
//                     cursor
//                         .add(i * NUM_TRANS_IN_UNIT + 1)
//                         .write((*seq.sampling_freq_div() - 1) as u16);
//                     cursor
//                         .add(i * NUM_TRANS_IN_UNIT + 2)
//                         .write((geometry.wavelength * 1000.0) as u16);
//                 }
//                 offset += 4;
//             }

//             let send_size = (seq.control_points().len() - *seq_sent).clamp(
//                 0,
//                 (EC_OUTPUT_FRAME_SIZE - HEADER_SIZE - offset * std::mem::size_of::<u16>())
//                     / std::mem::size_of::<SeqFocus>(),
//             );

//             if *seq_sent + send_size == seq.control_points().len() {
//                 (*header).cpu_flag |= CPUControlFlags::SEQ_END;
//                 (*header).fpga_flag |= FPGAControlFlags::OUTPUT_ENABLE;
//             }

//             let fixed_num_unit = 256.0 / geometry.wavelength;
//             for device in geometry.devices() {
//                 std::ptr::write(cursor, send_size as u16);
//                 let mut focus_cursor = cursor.add(offset) as *mut SeqFocus;
//                 for i in 0..send_size {
//                     let cp = seq.control_points()[*seq_sent + i];
//                     let v64 = device.local_position(cp.0) * fixed_num_unit;
//                     (*focus_cursor).set(v64[0] as i32, v64[1] as i32, v64[2] as i32, cp.1);
//                     focus_cursor = focus_cursor.add(1);
//                 }
//                 cursor = cursor.add(NUM_TRANS_IN_UNIT);
//             }
//             *seq_sent += send_size;
//         }

//         size
//     }
