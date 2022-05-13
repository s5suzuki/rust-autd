/*
 * File: gain.rs
 * Project: src
 * Created Date: 27/04/2022
 * Author: Shun Suzuki
 * -----
 * Last Modified: 13/05/2022
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2022 Hapis Lab. All rights reserved.
 *
 */

use autd3_driver::TxDatagram;

use crate::{
    geometry::{DriveData, Geometry, Transducer},
    interface::DatagramBody,
};
use anyhow::Result;

pub struct GainProps<T: Transducer> {
    pub built: bool,
    pub phase_sent: bool,
    pub duty_sent: bool,
    pub drives: T::D,
}

impl<T: Transducer> GainProps<T> {
    pub fn new() -> Self {
        Self {
            built: false,
            phase_sent: false,
            duty_sent: false,
            drives: T::D::new(),
        }
    }

    pub fn init(&mut self, size: usize) {
        self.drives.init(size);
    }

    pub fn pack_head(&mut self, msg_id: u8, tx: &mut TxDatagram) {
        T::pack_head(msg_id, tx);
    }

    pub fn pack_body(&mut self, tx: &mut TxDatagram) -> Result<()> {
        T::pack_body(&mut self.phase_sent, &mut self.duty_sent, &self.drives, tx)
    }
}

impl<T: Transducer> Default for GainProps<T> {
    fn default() -> Self {
        Self::new()
    }
}

pub trait IGain<T: Transducer> {
    fn calc(&mut self, geometry: &Geometry<T>) -> Result<()>;
}

/// Gain contains amplitude and phase of each transducer in the AUTD.
/// Note that the amplitude means duty ratio of Pulse Width Modulation, respectively.
pub trait Gain<T: Transducer>: IGain<T> + DatagramBody<T> {
    fn build(&mut self, geometry: &Geometry<T>) -> Result<()>;
    fn rebuild(&mut self, geometry: &Geometry<T>) -> Result<()>;
    fn drives(&self) -> &T::D;
    fn take_drives(self) -> T::D;
    fn built(&self) -> bool;
}
