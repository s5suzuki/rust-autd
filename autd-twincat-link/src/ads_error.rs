/*
 * File: ads_error.rs
 * Project: ruautd-twincat-link
 * Created Date: 16/12/2019
 * Author: Shun Suzuki
 * -----
 * Last Modified: 24/05/2020
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2019 Hapis Lab. All rights reserved.
 *
 */

use std::error;
use std::fmt;

#[derive(Debug)]
#[allow(dead_code)]
pub enum ADSError {
    FailedOpenPort,
    FailedGetLocalAddress(i32),
    FailedSendData(i32),
    FailedReadData(i32),
    AmsNetIdParseError,
    FailedConnectRemote,
    ErrorDeviceInvalidSize,
}

impl fmt::Display for ADSError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ADSError::FailedOpenPort => write!(f, "Failed to open a new ADS port"),
            ADSError::FailedGetLocalAddress(n_err) => {
                write!(f, "AdsGetLocalAddress (error code: {})", n_err)
            }
            ADSError::FailedSendData(n_err) => {
                write!(f, "Failed to send data (error code: {})", n_err)
            }
            ADSError::FailedReadData(n_err) => {
                write!(f, "Failed to read data (error code: {})", n_err)
            }
            ADSError::AmsNetIdParseError => write!(f, "Ams net id must have 6 octets"),
            ADSError::FailedConnectRemote => write!(f, "Could not connect to remote"),
            ADSError::ErrorDeviceInvalidSize => write!(f, "The number of devices is invalid"),
        }
    }
}

impl error::Error for ADSError {
    fn description(&self) -> &str {
        match *self {
            ADSError::FailedOpenPort => "Failed to open a new ADS port",
            ADSError::FailedGetLocalAddress(_) => "Failed to AdsGetLocalAddress",
            ADSError::FailedSendData(_) => "Failed to send data",
            ADSError::FailedReadData(_) => "Failed to read data",
            ADSError::AmsNetIdParseError => "Ams net id must have 6 octets",
            ADSError::FailedConnectRemote => "Could not connect to remote",
            ADSError::ErrorDeviceInvalidSize => "The number of devices is invalid",
        }
    }

    fn cause(&self) -> Option<&dyn error::Error> {
        match *self {
            ADSError::FailedOpenPort => None,
            ADSError::FailedGetLocalAddress(_) => None,
            ADSError::FailedSendData(_) => None,
            ADSError::FailedReadData(_) => None,
            ADSError::AmsNetIdParseError => None,
            ADSError::FailedConnectRemote => None,
            ADSError::ErrorDeviceInvalidSize => None,
        }
    }
}
