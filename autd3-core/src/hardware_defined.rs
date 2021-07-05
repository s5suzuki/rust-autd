/*
 * File: hardware_defined.rs
 * Project: src
 * Created Date: 24/05/2021
 * Author: Shun Suzuki
 * -----
 * Last Modified: 05/07/2021
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2021 Hapis Lab. All rights reserved.
 *
 */

pub const NUM_TRANS_IN_UNIT: usize = 249;
pub const NUM_TRANS_X: usize = 18;
pub const NUM_TRANS_Y: usize = 14;
pub const TRANS_SPACING_MM: f64 = 10.16;
pub const AUTD_WIDTH: f64 = 192.0;
pub const AUTD_HEIGHT: f64 = 151.4;

pub fn is_missing_transducer(x: usize, y: usize) -> bool {
    y == 1 && (x == 1 || x == 2 || x == 16)
}

pub const FPGA_CLOCK: usize = 20400000;
pub const ULTRASOUND_FREQUENCY: usize = 40000;

pub const MOD_BUF_SIZE_MAX: usize = 65536;
pub const MOD_SAMPLING_FREQ_BASE: f64 = 40000.0;
pub const MOD_FRAME_SIZE: usize = 124;

pub const POINT_SEQ_BUFFER_SIZE_MAX: usize = 65536;
pub const POINT_SEQ_CLK_IDX_MAX: usize = 40000;
pub const POINT_SEQ_BASE_FREQ: usize = 40000;

pub type DataArray = [u16; NUM_TRANS_IN_UNIT];

bitflags! {
    pub struct RxGlobalControlFlags : u8 {
        const NONE = 0;
        const MOD_BEGIN = 1;
        const MOD_END = 1 << 1;
        const READ_FPGA_INFO = 1 << 2;
        const SILENT = 1 << 3;
        const FORCE_FAN = 1 << 4;
        const SEQ_MODE = 1 << 5;
        const SEQ_BEGIN = 1 << 6;
        const SEQ_END = 1 << 7;
    }
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum CommandType {
    Op = 0x00,
    ReadCpuVerLsb = 0x02,
    ReadCpuVerMsb = 0x03,
    ReadFpgaVerLsb = 0x04,
    ReadFpgaVerMsb = 0x05,
    SeqMode = 0x06,
    Clear = 0x09,
    SetDelay = 0x0A,
    Pause = 0x0B,
    Resume = 0x0C,
    EmulatorSetGeometry = 0xFF,
}

#[repr(C)]
pub struct RxGlobalHeader {
    pub msg_id: u8,
    pub ctrl_flag: RxGlobalControlFlags,
    pub command: CommandType,
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
