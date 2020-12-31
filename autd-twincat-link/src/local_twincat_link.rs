/*
 * File: local_twincat_link.rs
 * Project: ruautd-twincat-link
 * Created Date: 16/12/2019
 * Author: Shun Suzuki
 * -----
 * Last Modified: 31/12/2020
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2019 Hapis Lab. All rights reserved.
 *
 */

use std::error::Error;

use libc::c_void;

use autd::link::Link;

use crate::ads_error::ADSError;
use crate::consts::*;
use crate::native_methods::*;

pub struct LocalTwinCATLink {
    port: i32,
    send_addr: AmsAddr,
}

impl LocalTwinCATLink {
    pub fn new() -> Self {
        unsafe {
            let ams_addr: AmsAddr = std::mem::zeroed();
            Self {
                port: 0,
                send_addr: AmsAddr {
                    net_id: ams_addr.net_id,
                    port: PORT,
                },
            }
        }
    }
}

impl Default for LocalTwinCATLink {
    fn default() -> Self {
        Self::new()
    }
}

impl Link for LocalTwinCATLink {
    fn open(&mut self) -> Result<(), Box<dyn Error>> {
        unsafe {
            let port = (TC_ADS.tc_ads_port_open)();
            if port == 0 {
                return Err(From::from(ADSError::FailedOpenPort));
            }

            let mut ams_addr: AmsAddr = std::mem::zeroed();
            let n_err = (TC_ADS.tc_ads_get_local_address)(port, &mut ams_addr as *mut _);
            if n_err != 0 {
                return Err(From::from(ADSError::FailedGetLocalAddress(n_err)));
            }
        }

        Ok(())
    }

    fn close(&mut self) -> Result<(), Box<dyn Error>> {
        self.port = 0;
        unsafe {
            (TC_ADS.tc_ads_port_close)(0);
        }
        Ok(())
    }

    fn send(&mut self, data: &[u8]) -> Result<(), Box<dyn Error>> {
        unsafe {
            let n_err = (TC_ADS.tc_ads_sync_write_req)(
                self.port,
                &self.send_addr as *const _,
                INDEX_GROUP,
                INDEX_OFFSET_BASE,
                data.len() as u32,
                data.as_ptr() as *const c_void,
            );

            if n_err > 0 {
                Err(From::from(ADSError::FailedSendData(n_err)))
            } else {
                Ok(())
            }
        }
    }

    fn read(&mut self, data: &mut [u8], buffer_len: usize) -> Result<(), Box<dyn Error>> {
        let mut read_bytes: u32 = 0;
        unsafe {
            let n_err = (TC_ADS.tc_ads_sync_read_req)(
                self.port,
                &self.send_addr as *const _,
                INDEX_GROUP,
                INDEX_OFFSET_BASE_READ,
                buffer_len as u32,
                data.as_mut_ptr() as *mut c_void,
                &mut read_bytes as *mut u32,
            );

            if n_err > 0 {
                Err(From::from(ADSError::FailedReadData(n_err)))
            } else {
                Ok(())
            }
        }
    }

    fn is_open(&self) -> bool {
        self.port > 0
    }
}
