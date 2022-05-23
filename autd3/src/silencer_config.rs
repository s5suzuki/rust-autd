/*
 * File: silencer_config.rs
 * Project: src
 * Created Date: 28/04/2022
 * Author: Shun Suzuki
 * -----
 * Last Modified: 23/05/2022
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2022 Hapis Lab. All rights reserved.
 *
 */

use anyhow::Result;
use autd3_core::{
    geometry::Transducer,
    interface::{DatagramHeader, Empty, Filled, Sendable},
};

pub struct SilencerConfig {
    pub(crate) step: u16,
    pub(crate) cycle: u16,
}

impl SilencerConfig {
    pub fn new(step: u16, cycle: u16) -> Self {
        SilencerConfig { step, cycle }
    }

    pub fn none() -> Self {
        Self::new(0xFFFF, 4096)
    }
}

impl DatagramHeader for SilencerConfig {
    fn init(&mut self) -> Result<()> {
        Ok(())
    }

    fn pack(&mut self, msg_id: u8, tx: &mut autd3_core::TxDatagram) -> Result<()> {
        autd3_core::config_silencer(msg_id, self.cycle, self.step, tx)
    }

    fn is_finished(&self) -> bool {
        true
    }
}

impl<T: Transducer> Sendable<T> for SilencerConfig {
    type H = Filled;
    type B = Empty;

    fn init(&mut self) -> Result<()> {
        DatagramHeader::init(self)
    }

    fn pack(
        &mut self,
        msg_id: u8,
        _geometry: &autd3_core::geometry::Geometry<T>,
        tx: &mut autd3_core::TxDatagram,
    ) -> Result<()> {
        DatagramHeader::pack(self, msg_id, tx)
    }

    fn is_finished(&self) -> bool {
        DatagramHeader::is_finished(self)
    }
}

impl Default for SilencerConfig {
    fn default() -> Self {
        Self::new(10, 4096)
    }
}
