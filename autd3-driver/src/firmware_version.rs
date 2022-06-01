/*
 * File: firmware_version.rs
 * Project: src
 * Created Date: 27/04/2022
 * Author: Shun Suzuki
 * -----
 * Last Modified: 01/06/2022
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2022 Shun Suzuki. All rights reserved.
 *
 */

use std::fmt;

const ENABLED_STM_BIT: u8 = 0x01;
const ENABLED_MODULATOR_BIT: u8 = 0x02;
const ENABLED_SILENCER_BIT: u8 = 0x04;
const ENABLED_MOD_DELAY_BIT: u8 = 0x08;

pub struct FirmwareInfo {
    idx: usize,
    cpu_version_number: u8,
    fpga_version_number: u8,
    fpga_function_bits: u8,
}

impl FirmwareInfo {
    pub fn new(
        idx: usize,
        cpu_version_number: u8,
        fpga_version_number: u8,
        fpga_function_bits: u8,
    ) -> Self {
        Self {
            idx,
            cpu_version_number,
            fpga_version_number,
            fpga_function_bits,
        }
    }

    pub fn cpu_version(&self) -> String {
        Self::firmware_version_map(self.cpu_version_number)
    }

    pub fn fpga_version(&self) -> String {
        Self::firmware_version_map(self.fpga_version_number)
    }

    pub fn stm_enabled(&self) -> bool {
        (self.fpga_function_bits & ENABLED_STM_BIT) == ENABLED_STM_BIT
    }

    pub fn modulator_enabled(&self) -> bool {
        (self.fpga_function_bits & ENABLED_MODULATOR_BIT) == ENABLED_MODULATOR_BIT
    }

    pub fn silencer_enabled(&self) -> bool {
        (self.fpga_function_bits & ENABLED_SILENCER_BIT) == ENABLED_SILENCER_BIT
    }

    pub fn modulation_delay_enabled(&self) -> bool {
        (self.fpga_function_bits & ENABLED_MOD_DELAY_BIT) == ENABLED_MOD_DELAY_BIT
    }

    fn firmware_version_map(version_number: u8) -> String {
        match version_number {
            0 => "older than v0.4".to_string(),
            0x01..=0x06 => format!("v0.{}", version_number + 3),
            0x0A..=0x15 => format!("v1.{}", version_number - 0x0A),
            0x80..=0x81 => format!("v2.{}", version_number - 0x80),
            0xFF => "emulator".to_string(),
            _ => format!("unknown: {}", version_number),
        }
    }
}

impl fmt::Display for FirmwareInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            r"{}: CPU = {}, FPGA = {} (STM = {}, Modulator = {}, Silencer = {}, ModDelay = {})",
            self.idx,
            self.cpu_version(),
            self.fpga_version(),
            self.stm_enabled(),
            self.modulator_enabled(),
            self.silencer_enabled(),
            self.modulation_delay_enabled()
        )
    }
}
