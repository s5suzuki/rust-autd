/*
 * File: interface.rs
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
    geometry::Geometry,
    hardware_defined::{CPUControlFlags, FPGAControlFlags, TxDatagram},
};

pub trait IDatagramHeader {
    fn init(&mut self);
    fn pack(
        &mut self,
        msg_id: u8,
        tx: &mut TxDatagram,
        fpga_flag: FPGAControlFlags,
        cpu_flag: CPUControlFlags,
    );
    fn is_finished(&self) -> bool;
}

pub trait IDatagramBody {
    fn init(&mut self); 
    fn pack(&mut self, geometry: &Geometry, tx: &mut TxDatagram);
    fn is_finished(&self) -> bool;
}
