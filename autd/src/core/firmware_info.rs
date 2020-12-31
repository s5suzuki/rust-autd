/*
 * File: firmware_info.rs
 * Project: src
 * Created Date: 04/04/2020
 * Author: Shun Suzuki
 * -----
 * Last Modified: 31/12/2020
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2020 Hapis Lab. All rights reserved.
 *
 */

use std::fmt;

pub struct FirmwareInfo {
    idx: u16,
    cpu_version_number: u16,
    fpga_version_number: u16,
}

impl FirmwareInfo {
    pub fn new(idx: u16, cpu_version_number: u16, fpga_version_number: u16) -> FirmwareInfo {
        FirmwareInfo {
            idx,
            cpu_version_number,
            fpga_version_number,
        }
    }

    pub fn cpu_version(&self) -> String {
        FirmwareInfo::firmware_version_map(self.cpu_version_number)
    }

    pub fn fpga_version(&self) -> String {
        FirmwareInfo::firmware_version_map(self.fpga_version_number)
    }

    fn firmware_version_map(version_number: u16) -> String {
        match version_number {
            0 => "older than v0.4".to_string(),
            1 => "v0.4".to_string(),
            2 => "v0.5".to_string(),
            3 => "v0.6".to_string(),
            4 => "v0.7".to_string(),
            5 => "v0.8".to_string(),
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
