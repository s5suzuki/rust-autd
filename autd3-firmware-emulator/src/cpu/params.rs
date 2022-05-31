/*
 * File: params.rs
 * Project: cpu
 * Created Date: 07/05/2022
 * Author: Shun Suzuki
 * -----
 * Last Modified: 31/05/2022
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2022 Shun Suzuki. All rights reserved.
 *
 */

pub const CPU_VERSION: u16 = 0x80;

pub const BRAM_SELECT_CONTROLLER: u8 = 0x0;
pub const BRAM_SELECT_MOD: u8 = 0x1;
pub const BRAM_SELECT_NORMAL: u8 = 0x2;
pub const BRAM_SELECT_STM: u8 = 0x3;

pub const BRAM_ADDR_CTL_REG: u16 = 0x000;
pub const BRAM_ADDR_FPGA_INFO: u16 = 0x001;
pub const BRAM_ADDR_EC_SYNC_CYCLE_TICKS: u16 = 0x010;
// pub const BRAM_ADDR_EC_SYNC_TIME_0: u16 = BRAM_ADDR_EC_SYNC_CYCLE_TICKS + 1;
// pub const BRAM_ADDR_EC_SYNC_TIME_1: u16 = BRAM_ADDR_EC_SYNC_CYCLE_TICKS + 2;
// pub const BRAM_ADDR_EC_SYNC_TIME_2: u16 = BRAM_ADDR_EC_SYNC_CYCLE_TICKS + 3;
// pub const BRAM_ADDR_EC_SYNC_TIME_3: u16 = BRAM_ADDR_EC_SYNC_CYCLE_TICKS + 4;
pub const BRAM_ADDR_MOD_ADDR_OFFSET: u16 = 0x020;
pub const BRAM_ADDR_MOD_CYCLE: u16 = 0x021;
pub const BRAM_ADDR_MOD_FREQ_DIV_0: u16 = 0x022;
pub const BRAM_ADDR_VERSION_NUM: u16 = 0x03F;
pub const BRAM_ADDR_SILENT_CYCLE: u16 = 0x040;
pub const BRAM_ADDR_SILENT_STEP: u16 = 0x041;
pub const BRAM_ADDR_STM_ADDR_OFFSET: u16 = 0x050;
pub const BRAM_ADDR_STM_CYCLE: u16 = 0x051;
pub const BRAM_ADDR_STM_FREQ_DIV_0: u16 = 0x052;
pub const BRAM_ADDR_SOUND_SPEED_0: u16 = 0x054;
pub const BRAM_ADDR_CYCLE_BASE: u16 = 0x100;

pub const MOD_BUF_SEGMENT_SIZE_WIDTH: u32 = 15;
pub const MOD_BUF_SEGMENT_SIZE: u32 = 1 << MOD_BUF_SEGMENT_SIZE_WIDTH;
pub const MOD_BUF_SEGMENT_SIZE_MASK: u32 = MOD_BUF_SEGMENT_SIZE - 1;
pub const POINT_STM_BUF_SEGMENT_SIZE_WIDTH: u32 = 11;
pub const POINT_STM_BUF_SEGMENT_SIZE: u32 = 1 << POINT_STM_BUF_SEGMENT_SIZE_WIDTH;
pub const POINT_STM_BUF_SEGMENT_SIZE_MASK: u32 = POINT_STM_BUF_SEGMENT_SIZE - 1;
pub const GAIN_STM_BUF_SEGMENT_SIZE_WIDTH: u32 = 5;
pub const GAIN_STM_BUF_SEGMENT_SIZE: u32 = 1 << GAIN_STM_BUF_SEGMENT_SIZE_WIDTH;
pub const GAIN_STM_BUF_SEGMENT_SIZE_MASK: u32 = GAIN_STM_BUF_SEGMENT_SIZE - 1;
