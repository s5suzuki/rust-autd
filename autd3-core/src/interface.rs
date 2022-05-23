/*
 * File: interface.rs
 * Project: src
 * Created Date: 27/04/2022
 * Author: Shun Suzuki
 * -----
 * Last Modified: 23/05/2022
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2022 Hapis Lab. All rights reserved.
 *
 */

use autd3_driver::TxDatagram;

use crate::geometry::{Geometry, LegacyTransducer, NormalTransducer, Transducer};
use anyhow::Result;

pub trait DatagramHeader {
    fn init(&mut self) -> Result<()>;
    fn pack(&mut self, msg_id: u8, tx: &mut TxDatagram) -> Result<()>;
    fn is_finished(&self) -> bool;
}

pub trait DatagramBody<T: Transducer> {
    fn init(&mut self) -> Result<()>;
    fn pack(&mut self, geometry: &Geometry<T>, tx: &mut TxDatagram) -> Result<()>;
    fn is_finished(&self) -> bool;
}

pub struct NullHeader {}

impl DatagramHeader for NullHeader {
    fn init(&mut self) -> Result<()> {
        Ok(())
    }

    fn pack(&mut self, msg_id: u8, tx: &mut TxDatagram) -> Result<()> {
        autd3_driver::null_header(msg_id, tx);
        Ok(())
    }

    fn is_finished(&self) -> bool {
        true
    }
}

pub struct NullBody {}

impl DatagramBody<LegacyTransducer> for NullBody {
    fn init(&mut self) -> Result<()> {
        Ok(())
    }

    fn pack(&mut self, geometry: &Geometry<LegacyTransducer>, tx: &mut TxDatagram) -> Result<()> {
        autd3_driver::null_body(tx);
        Ok(())
    }

    fn is_finished(&self) -> bool {
        true
    }
}

impl DatagramBody<NormalTransducer> for NullBody {
    fn init(&mut self) -> Result<()> {
        Ok(())
    }

    fn pack(&mut self, _geometry: &Geometry<NormalTransducer>, tx: &mut TxDatagram) -> Result<()> {
        autd3_driver::null_body(tx);
        Ok(())
    }

    fn is_finished(&self) -> bool {
        true
    }
}
