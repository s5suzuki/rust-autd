/*
* File: datagrams.rs
* Project: src
* Created Date: 16/12/2021
* Author: Shun Suzuki
* -----
* Last Modified: 16/12/2021
* Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
* -----
* Copyright (c) 2021 Hapis Lab. All rights reserved.
*
*/

use crate::{
    hardware_defined::FPGAControlFlags,
    interface::{IDatagramBody, IDatagramHeader},
};
use anyhow::Result;

pub struct CommonHeader {
    fpga_mask: FPGAControlFlags,
}

impl CommonHeader {
    pub fn new(fpga_mask: FPGAControlFlags) -> Self {
        Self { fpga_mask }
    }
}

impl IDatagramHeader for CommonHeader {
    fn init(&mut self) -> Result<()> {
        Ok(())
    }

    fn pack(
        &mut self,
        msg_id: u8,
        tx: &mut crate::hardware_defined::TxDatagram,
        _fpga_flag: FPGAControlFlags,
        cpu_flag: crate::hardware_defined::CPUControlFlags,
    ) {
        let mut header = tx.header_mut();
        header.msg_id = msg_id;
        header.fpga_flag =
            (header.fpga_flag & !self.fpga_mask) | (header.fpga_flag & self.fpga_mask);
        header.cpu_flag = cpu_flag;
        header.mod_size = 0;
        tx.set_num_bodies(0);
    }

    fn is_finished(&self) -> bool {
        true
    }
}

pub struct SpecialMessageIdHeader {
    msg_id: u8,
    fpga_mask: FPGAControlFlags,
}

impl SpecialMessageIdHeader {
    pub fn new(msg_id: u8, fpga_mask: FPGAControlFlags) -> Self {
        Self { msg_id, fpga_mask }
    }
}

impl IDatagramHeader for SpecialMessageIdHeader {
    fn init(&mut self) -> Result<()> {
        Ok(())
    }

    fn pack(
        &mut self,
        _msg_id: u8,
        tx: &mut crate::hardware_defined::TxDatagram,
        _fpga_flag: FPGAControlFlags,
        cpu_flag: crate::hardware_defined::CPUControlFlags,
    ) {
        let mut header = tx.header_mut();
        header.msg_id = self.msg_id;
        header.fpga_flag =
            (header.fpga_flag & !self.fpga_mask) | (header.fpga_flag & self.fpga_mask);
        header.cpu_flag = cpu_flag;
        header.mod_size = 0;
        tx.set_num_bodies(0);
    }

    fn is_finished(&self) -> bool {
        true
    }
}

pub struct NullBody {}

impl NullBody {
    pub fn new() -> Self {
        Self {}
    }
}

impl IDatagramBody for NullBody {
    fn init(&mut self) {}

    fn pack(
        &mut self,
        _geometry: &crate::geometry::Geometry,
        tx: &mut crate::hardware_defined::TxDatagram,
    ) -> Result<()> {
        tx.set_num_bodies(0);
        Ok(())
    }

    fn is_finished(&self) -> bool {
        true
    }
}

impl Default for NullBody {
    fn default() -> Self {
        Self::new()
    }
}
