/*
 * File: params.rs
 * Project: fpga
 * Created Date: 07/05/2022
 * Author: Shun Suzuki
 * -----
 * Last Modified: 31/05/2022
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2022 Shun Suzuki. All rights reserved.
 *
 */

pub const VERSION_NUM: u8 = 0x80;

pub const BRAM_SELECT_CONTROLLER: u16 = 0x0;
pub const BRAM_SELECT_MOD: u16 = 0x1;
pub const BRAM_SELECT_NORMAL: u16 = 0x2;
pub const BRAM_SELECT_STM: u16 = 0x3;

pub const ADDR_CTL_REG: usize = 0x0000;
// pub const ADDR_FPGA_INFO: usize = 0x0001;
pub const ADDR_EC_SYNC_CYCLE_TICKS: usize = 0x0010;
// pub const ADDR_EC_SYNC_TIME_0: usize = ADDR_EC_SYNC_CYCLE_TICKS + 1;
// pub const ADDR_EC_SYNC_TIME_1: usize = ADDR_EC_SYNC_CYCLE_TICKS + 2;
// pub const ADDR_EC_SYNC_TIME_2: usize = ADDR_EC_SYNC_CYCLE_TICKS + 3;
// pub const ADDR_EC_SYNC_TIME_3: usize = ADDR_EC_SYNC_CYCLE_TICKS + 4;
pub const ADDR_MOD_ADDR_OFFSET: usize = 0x0020;
pub const ADDR_MOD_CYCLE: usize = 0x0021;
pub const ADDR_MOD_FREQ_DIV_0: usize = 0x0022;
pub const ADDR_MOD_FREQ_DIV_1: usize = 0x0023;
pub const ADDR_VERSION_NUM: usize = 0x003F;
pub const ADDR_SILENT_CYCLE: usize = 0x0040;
pub const ADDR_SILENT_STEP: usize = 0x0041;
pub const ADDR_STM_ADDR_OFFSET: usize = 0x0050;
pub const ADDR_STM_CYCLE: usize = 0x0051;
pub const ADDR_STM_FREQ_DIV_0: usize = 0x0052;
pub const ADDR_STM_FREQ_DIV_1: usize = 0x0053;
pub const ADDR_SOUND_SPEED_0: usize = 0x0054;
pub const ADDR_SOUND_SPEED_1: usize = 0x0055;
pub const ADDR_CYCLE_BASE: usize = 0x0100;

pub const CTL_REG_LEGACY_MODE_BIT: u16 = 0;
pub const CTL_REG_FORCE_FAN_BIT: u16 = 4;
pub const CTL_REG_OP_MODE_BIT: u16 = 5;
pub const CTL_REG_STM_GAIN_MODE_BIT: u16 = 6;
// pub const CTL_REG_SYNC_BIT: usize = 8;

pub const ENABLED_STM_BIT: u8 = 0x01;
pub const ENABLED_MODULATOR_BIT: u8 = 0x02;
pub const ENABLED_SILENCER_BIT: u8 = 0x04;
pub const ENABLED_FEATURES_BITS: u8 =
    ENABLED_STM_BIT | ENABLED_MODULATOR_BIT | ENABLED_SILENCER_BIT;
