/*
* File: delay_offset.rs
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
    hardware_defined::{CPUControlFlags, DelayOffset, NUM_TRANS_IN_UNIT},
    interface::IDatagramBody,
};

pub struct DelayOffsets {
    data: Vec<DelayOffset>,
}

impl DelayOffsets {
    pub fn new(num_devices: usize) -> Self {
        Self {
            data: vec![DelayOffset::new(); num_devices * NUM_TRANS_IN_UNIT],
        }
    }
}

impl IDatagramBody for DelayOffsets {
    fn init(&mut self) {}

    fn pack(
        &mut self,
        geometry: &crate::geometry::Geometry,
        tx: &mut crate::hardware_defined::TxDatagram,
    ) {
        let header = tx.header();
        header.cpu_flag |= CPUControlFlags::DELAY_OFFSET;
        header.cpu_flag |= CPUControlFlags::WRITE_BODY;
        for (dst, &src) in tx
            .body_data::<DelayOffset>()
            .iter_mut()
            .zip(self.data.iter())
        {
            *dst = src;
        }
        tx.set_num_bodies(geometry.num_devices());
    }

    fn is_finished(&self) -> bool {
        true
    }
}
