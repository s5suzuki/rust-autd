/*
 * File: interface.rs
 * Project: src
 * Created Date: 27/04/2022
 * Author: Shun Suzuki
 * -----
 * Last Modified: 06/05/2022
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2022 Hapis Lab. All rights reserved.
 *
 */

use autd3_driver::TxDatagram;

use crate::geometry::{Geometry, Transducer};
use anyhow::Result;

pub struct Empty;
pub struct Filled;

pub trait Sendable<T: Transducer> {
    type H;
    type B;
    fn init(&mut self) -> Result<()>;
    fn pack(&mut self, msg_id: u8, geometry: &Geometry<T>, tx: &mut TxDatagram) -> Result<()>;
    fn is_finished(&self) -> bool;
}

pub trait DatagramHeader {
    fn init(&mut self) -> Result<()>;
    fn pack(&mut self, msg_id: u8, tx: &mut TxDatagram) -> Result<()>;
    fn is_finished(&self) -> bool;
}

pub trait DatagramBody<T: Transducer> {
    fn init(&mut self) -> Result<()>;
    fn pack(&mut self, msg_id: u8, geometry: &Geometry<T>, tx: &mut TxDatagram) -> Result<()>;
    fn is_finished(&self) -> bool;
}
