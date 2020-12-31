/*
 * File: remote_twincat_link.rs
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

pub struct RemoteTwinCATLink {
    port: i32,
    send_addr: AmsAddr,
    ams_net_id: String,
    ipv4_addr: String,
}

impl RemoteTwinCATLink {
    pub fn new(ams_net_id: &str) -> Self {
        Self::new_with_ipaddr("", ams_net_id)
    }

    pub fn new_with_ipaddr(ipv4_addr: &str, ams_net_id: &str) -> Self {
        unsafe {
            let ams_addr: AmsAddr = std::mem::zeroed();
            Self {
                port: 0,
                send_addr: AmsAddr {
                    net_id: ams_addr.net_id,
                    port: PORT,
                },
                ams_net_id: ams_net_id.to_string(),
                ipv4_addr: ipv4_addr.to_string(),
            }
        }
    }
}

impl Link for RemoteTwinCATLink {
    fn open(&mut self) -> Result<(), Box<dyn Error>> {
        unsafe {
            let ams_net_id = &self.ams_net_id;
            let ipv4addr = &self.ipv4_addr;
            let octets: Vec<&str> = ams_net_id.split('.').collect();
            if octets.len() != 6 {
                return Err(From::from(ADSError::AmsNetIdParseError));
            }

            let addr = if ipv4addr == "" {
                octets[0..4].join(".")
            } else {
                ipv4addr.to_string()
            };

            let net_id = AmsNetId {
                b: [
                    octets[0].parse().unwrap(),
                    octets[1].parse().unwrap(),
                    octets[2].parse().unwrap(),
                    octets[3].parse().unwrap(),
                    octets[4].parse().unwrap(),
                    octets[5].parse().unwrap(),
                ],
            };

            let c_addr = std::ffi::CString::new(addr).unwrap();
            if AdsAddRoute(net_id, c_addr.as_ptr()) != 0 {
                return Err(From::from(ADSError::FailedConnectRemote));
            }
            let port = AdsPortOpenEx();
            if port == 0 {
                return Err(From::from(ADSError::FailedOpenPort));
            }

            Ok(())
        }
    }

    fn close(&mut self) -> Result<(), Box<dyn Error>> {
        self.port = 0;
        unsafe {
            AdsPortCloseEx(0);
        }
        Ok(())
    }

    fn send(&mut self, data: &[u8]) -> Result<(), Box<dyn Error>> {
        unsafe {
            let n_err = AdsSyncWriteReqEx(
                self.port,
                &self.send_addr as *const _,
                INDEX_GROUP,
                INDEX_OFFSET_BASE,
                data.len() as u32,
                data.as_ptr() as *const c_void,
            );

            if n_err > 0 {
                if n_err == ADSERR_DEVICE_INVALIDSIZE {
                    Err(From::from(ADSError::ErrorDeviceInvalidSize))
                } else {
                    Err(From::from(ADSError::FailedSendData(n_err)))
                }
            } else {
                Ok(())
            }
        }
    }

    fn read(&mut self, data: &mut [u8], buffer_len: usize) -> Result<(), Box<dyn Error>> {
        let mut read_bytes: u32 = 0;
        unsafe {
            let n_err = AdsSyncReadReqEx2(
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
