/*
 * File: hardware_defined.rs
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

pub type DataArray = [u16; NUM_TRANS_IN_UNIT];

bitflags! {
    pub struct FPGAControlFlags : u8 {
        const NONE = 0;
        const OUTPUT_ENABLE = 1 << 0;
        const OUTPUT_BALANCE = 1 << 1;
        const SILENT = 1 << 3;
        const FORCE_FAN = 1 << 4;
        const OP_MODE = 1 << 5;
        const SEQ_MODE = 1 << 6;
    }
}

pub const OP_MODE_NORMAL: bool = false;
pub const OP_MODE_SEQ: bool = true;
pub const SEQ_MODE_POINT: bool = false;
pub const SEQ_MODE_GAIN: bool = true;

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

#[repr(C)]
pub(crate) struct SeqFocus {
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

#[repr(u16)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum GainMode {
    DutyPhaseFull = 1,
    PhaseFull = 2,
    PhaseHalf = 4,
}
