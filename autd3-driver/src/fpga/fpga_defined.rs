/*
 * File: fpga_defined.rs
 * Project: src
 * Created Date: 02/05/2022
 * Author: Shun Suzuki
 * -----
 * Last Modified: 01/06/2022
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2022 Shun Suzuki. All rights reserved.
 *
 */

use std::f64::consts::PI;

pub const FPGA_CLK_FREQ: usize = 163840000;

pub const MAX_CYCLE: u16 = 8191;

pub const MOD_SAMPLING_FREQ_DIV_MIN: u32 = 2320;
pub const MOD_BUF_SIZE_MAX: usize = 65536;

pub const POINT_STM_FIXED_NUM_UNIT: f64 = 0.025; //mm

pub const STM_SAMPLING_FREQ_DIV_MIN: u32 = 3224;
pub const POINT_STM_BUF_SIZE_MAX: usize = 65536;
pub const GAIN_STM_BUF_SIZE_MAX: usize = 1024;

pub const SILENCER_CYCLE_MIN: u16 = 2088;

bitflags::bitflags! {
    pub struct FPGAControlFlags : u8 {
        const NONE            = 0;
        const LEGACY_MODE     = 1 << 0;
        const FORCE_FAN       = 1 << 4;
        const STM_MODE        = 1 << 5;
        const STM_GAIN_MODE   = 1 << 6;
        const READS_FPGA_INFO = 1 << 7;
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct LegacyDrive {
    pub phase: u8,
    pub duty: u8,
}

impl LegacyDrive {
    pub fn set(&mut self, amp: f64, phase: f64) {
        self.duty = (510.0 * amp.asin() / PI) as u8;
        self.phase = (((phase * 256.0).round() as i32) & 0xFF) as u8;
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct Phase {
    pub phase: u16,
}

impl Phase {
    pub fn set(&mut self, phase: f64, cycle: u16) {
        self.phase = ((phase * cycle as f64).round() as i32).rem_euclid(cycle as i32) as _;
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct Duty {
    pub duty: u16,
}

impl Duty {
    pub fn set(&mut self, amp: f64, cycle: u16) {
        self.duty = (cycle as f64 * amp.asin() / PI) as _;
    }
}

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
}

impl Default for FPGAInfo {
    fn default() -> Self {
        Self::new()
    }
}
