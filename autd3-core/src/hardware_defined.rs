/*
* File: hardware_defined.rs
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

use std::mem::size_of;

pub const NUM_TRANS_IN_UNIT: usize = 249;
pub const NUM_TRANS_X: usize = 18;
pub const NUM_TRANS_Y: usize = 14;
pub const TRANS_SPACING_MM: f64 = 10.16;
pub const DEVICE_WIDTH: f64 = 192.0;
pub const DEVICE_HEIGHT: f64 = 151.4;

pub fn is_missing_transducer(x: usize, y: usize) -> bool {
    y == 1 && (x == 1 || x == 2 || x == 16)
}

pub const FPGA_CLOCK: usize = 20480000;
pub const ULTRASOUND_FREQUENCY: usize = 40000;

pub const MOD_BUF_SIZE_MAX: usize = 65536;
pub const MOD_SAMPLING_FREQ_BASE: f64 = 40000.0;
pub const MOD_SAMPLING_FREQ_DIV_MAX: usize = 65536;
pub const MOD_FRAME_SIZE: usize = 124;

pub const POINT_SEQ_BUFFER_SIZE_MAX: usize = 65536;
pub const GAIN_SEQ_BUFFER_SIZE_MAX: usize = 2048;
pub const SEQ_BASE_FREQ: usize = 40000;
pub const SEQ_SAMPLING_FREQ_DIV_MAX: usize = 65536;

bitflags! {
    pub struct FPGAControlFlags : u8 {
        const NONE = 0;
        const OUTPUT_ENABLE = 1 << 0;
        const OUTPUT_BALANCE = 1 << 1;
        const SILENT = 1 << 3;
        const FORCE_FAN = 1 << 4;
        const SEQ_MODE = 1 << 5;
        const SEQ_GAIN_MODE = 1 << 6;
    }
}

bitflags! {
    pub struct CPUControlFlags : u8 {
        const NONE = 0;
        const MOD_BEGIN = 1 << 0;
        const MOD_END = 1 << 1;
        const SEQ_BEGIN = 1 << 2;
        const SEQ_END = 1 << 3;
        const READS_FPGA_INFO = 1 << 4;
        const DELAY_OFFSET = 1 << 5;
        const WRITE_BODY = 1 << 6;
        const WAIT_ON_SYNC = 1 << 7;
    }
}

pub const MSG_CLEAR: u8 = 0x00;
pub const MSG_RD_CPU_V_LSB: u8 = 0x01;
pub const MSG_RD_CPU_V_MSB: u8 = 0x02;
pub const MSG_RD_FPGA_V_LSB: u8 = 0x03;
pub const MSG_RD_FPGA_V_MSB: u8 = 0x04;
pub const MSG_EMU_GEOMETRY_SET: u8 = 0x05;
pub const MSG_NORMAL_BASE: u8 = 0x06;

#[repr(C)]
pub struct GlobalHeader {
    pub msg_id: u8,
    pub fpga_flag: FPGAControlFlags,
    pub cpu_flag: CPUControlFlags,
    pub mod_size: u8,
    pub mod_data: [u8; MOD_FRAME_SIZE],
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Drive {
    pub phase: u8,
    pub duty: u8,
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct DelayOffset {
    pub delay: u8,
    pub offset: u8,
}

impl DelayOffset {
    pub fn new() -> Self {
        Self {
            delay: 0x00,
            offset: 0x01,
        }
    }
}

impl Default for DelayOffset {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct RxMessage {
    pub ack: u8,
    pub msg_id: u8,
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct FPGAInfo {
    info: u8,
}

impl FPGAInfo {
    pub fn new() -> Self {
        Self { info: 0 }
    }
    pub fn is_fan_running(&self) -> bool {
        (self.info & 0x01) != 0
    }

    pub fn copy_from(&mut self, rx: &RxMessage) {
        self.info = rx.ack;
    }
}

impl Default for FPGAInfo {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone)]
pub struct TxDatagram {
    data: Vec<u8>,
    header_size: usize,
    num_bodies: usize,
    body_size: usize,
}

impl TxDatagram {
    pub fn new(device_num: usize) -> Self {
        let header_size = std::mem::size_of::<GlobalHeader>();
        let body_size = std::mem::size_of::<Drive>() * NUM_TRANS_IN_UNIT;
        let num_bodies = device_num;
        Self {
            data: vec![0x00; header_size * num_bodies * body_size],
            header_size,
            num_bodies,
            body_size,
        }
    }

    pub fn data(&self) -> &[u8] {
        &self.data[0..(self.header_size + self.num_bodies * self.body_size)]
    }

    pub fn data_mut(&mut self) -> &mut [u8] {
        &mut self.data[0..(self.header_size + self.num_bodies * self.body_size)]
    }

    pub fn header(&self) -> &GlobalHeader {
        unsafe { (self.data.as_ptr() as *mut GlobalHeader).as_mut().unwrap() }
    }

    pub fn header_mut(&mut self) -> &mut GlobalHeader {
        unsafe {
            (self.data.as_mut_ptr() as *mut GlobalHeader)
                .as_mut()
                .unwrap()
        }
    }

    pub fn body_data<T>(&self) -> &[T] {
        unsafe {
            std::slice::from_raw_parts(
                self.data.as_ptr().add(self.header_size) as *mut T,
                self.data.len() / size_of::<T>(),
            )
        }
    }

    pub fn body_data_mut<T>(&mut self) -> &mut [T] {
        unsafe {
            std::slice::from_raw_parts_mut(
                self.data.as_mut_ptr().add(self.header_size) as *mut T,
                self.data.len() / size_of::<T>(),
            )
        }
    }

    pub fn num_bodies(&self) -> usize {
        self.num_bodies
    }

    pub fn set_num_bodies(&mut self, num_bodies: usize) {
        self.num_bodies = num_bodies;
    }

    pub fn copy_from(&mut self, other: &TxDatagram) {
        self.header_size = other.header_size;
        self.num_bodies = other.num_bodies();
        self.data.copy_from_slice(&other.data);
    }
}

pub struct RxDatagram {
    data: Vec<RxMessage>,
}

impl RxDatagram {
    pub fn new(num_devices: usize) -> Self {
        Self {
            data: vec![RxMessage { msg_id: 0, ack: 0 }; num_devices],
        }
    }

    pub fn messages(&self) -> &[RxMessage] {
        &self.data
    }
    pub fn messages_mut(&mut self) -> &mut [RxMessage] {
        &mut self.data
    }

    pub fn copy_from(&mut self, other: &[RxMessage]) {
        self.data.copy_from_slice(other);
    }
}

pub fn is_msg_processed(msg_id: u8, rx: &RxDatagram) -> bool {
    rx.messages().iter().all(|msg| msg.msg_id == msg_id)
}
