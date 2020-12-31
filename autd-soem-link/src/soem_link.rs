/*
 * File: soem_link.rs
 * Project: src
 * Created Date: 02/09/2019
 * Author: Shun Suzuki
 * -----
 * Last Modified: 31/12/2020
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2019 Hapis Lab. All rights reserved.
 *
 */

use std::error::Error;

use autd::consts::*;
use autd::link::Link;

use crate::soem_error::SOEMError;
use crate::soem_handler::{ECConfig, RuSOEM};

const EC_SM2_CYCTIME_NS: u32 = 1_000_000;
const EC_SYNC0_CYCTIME_NS: u32 = 1_000_000;

pub struct SoemLink {
    handler: RuSOEM,
    dev_num: u16,
    ec_sync0_cyctime_ns: u32,
    ec_sm2_cyctime_ns: u32,
}

impl SoemLink {
    pub fn new(ifname: &str, dev_num: u16) -> Self {
        let config = ECConfig {
            input_frame_size: INPUT_FRAME_SIZE,
            body_size: BODY_SIZE,
            header_size: HEADER_SIZE,
        };
        let handler = RuSOEM::new(ifname, config);
        Self {
            handler,
            dev_num,
            ec_sm2_cyctime_ns: EC_SM2_CYCTIME_NS,
            ec_sync0_cyctime_ns: EC_SYNC0_CYCTIME_NS,
        }
    }

    pub fn set_sm2_cyctime(&mut self, cyctime_ns: u32) {
        self.ec_sm2_cyctime_ns = cyctime_ns;
    }
}

impl Link for SoemLink {
    fn open(&mut self) -> Result<(), Box<dyn Error>> {
        self.handler.start(
            self.dev_num,
            self.ec_sm2_cyctime_ns,
            self.ec_sync0_cyctime_ns,
        )?;

        Ok(())
    }

    fn close(&mut self) -> Result<(), Box<dyn Error>> {
        self.handler.close();
        Ok(())
    }

    fn send(&mut self, data: &[u8]) -> Result<(), Box<dyn Error>> {
        self.handler.send(data);
        Ok(())
    }

    fn read(&mut self, data: &mut [u8], _buffer_len: usize) -> Result<(), Box<dyn Error>> {
        if self.handler.read(data) {
            Ok(())
        } else {
            Err(Box::new(SOEMError::FailedReadData))
        }
    }

    fn is_open(&self) -> bool {
        self.handler.is_open()
    }

    fn calibrate(&mut self) -> Result<bool, Box<dyn Error>> {
        unreachable!("This method is no longer necessary after v0.5")
    }
}
