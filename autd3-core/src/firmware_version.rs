/*
 * File: firmware_version.rs
 * Project: src
 * Created Date: 24/05/2021
 * Author: Shun Suzuki
 * -----
 * Last Modified: 21/07/2021
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2021 Hapis Lab. All rights reserved.
 *
 */

use std::fmt;

pub struct FirmwareInfo {
    idx: u16,
    cpu_version_number: u16,
    fpga_version_number: u16,
}

impl FirmwareInfo {
    pub fn new(idx: u16, cpu_version_number: u16, fpga_version_number: u16) -> Self {
        Self {
            idx,
            cpu_version_number,
            fpga_version_number,
        }
    }

    pub fn cpu_version(&self) -> String {
        Self::firmware_version_map(self.cpu_version_number)
    }

    pub fn fpga_version(&self) -> String {
        Self::firmware_version_map(self.fpga_version_number)
    }

    fn firmware_version_map(version_number: u16) -> String {
        match version_number {
            0 => "older than v0.4".to_string(),
            0x01..=0x06 => format!("v0.{}", version_number + 3),
            0x0A..=0x10 => format!("v1.{}", version_number - 0x0A),
            0xFFFF => "emulator".to_string(),
            _ => format!("unknown: {}", version_number),
        }
    }
}

impl fmt::Display for FirmwareInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            r"{}: CPU = {}, FPGA = {}",
            self.idx,
            self.cpu_version(),
            self.fpga_version()
        )
    }
}
