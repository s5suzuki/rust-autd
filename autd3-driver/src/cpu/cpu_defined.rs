/*
 * File: cpu_defined.rs
 * Project: src
 * Created Date: 02/05/2022
 * Author: Shun Suzuki
 * -----
 * Last Modified: 05/05/2022
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2022 Hapis Lab. All rights reserved.
 *
 */

pub const MSG_CLEAR: u8 = 0x00;
pub const MSG_RD_CPU_VERSION: u8 = 0x01;
pub const MSG_RD_FPGA_VERSION: u8 = 0x03;
pub const MSG_RD_FPGA_FUNCTION: u8 = 0x04;
pub const MSG_NORMAL_BASE: u8 = 0x05;

pub const MOD_HEAD_DATA_SIZE: usize = 120;
pub const MOD_BODY_DATA_SIZE: usize = 124;

pub const POINT_STM_HEAD_DATA_SIZE: usize = 61;
pub const POINT_STM_BODY_DATA_SIZE: usize = 62;

bitflags::bitflags! {
    pub struct CPUControlFlags : u8 {
        const NONE            = 0;
        const MOD_BEGIN       = 1 << 0;
        const MOD_END         = 1 << 1;
        const STM_BEGIN       = 1 << 2;
        const STM_END         = 1 << 3;
        const IS_DUTY         = 1 << 4;
        const CONFIG_SILENCER = 1 << 5;
        const READS_FPGA_INFO = 1 << 6;
        const DO_SYNC         = 1 << 7;
    }
}
