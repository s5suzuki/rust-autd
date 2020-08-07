/*
 * File: soem_error.rs
 * Project: src
 * Created Date: 21/11/2019
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
pub enum SOEMError {
    NoSocketConnection(String),
    SlaveNotFound(u16, u16),
    NotResponding,
    FailedReadData,
    CreateTimerError,
    DeleteTimerError,
}

impl fmt::Display for SOEMError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::NoSocketConnection(ref ifname) => write!(f, "No socket connection on {}", ifname),
            Self::SlaveNotFound(wc, num) => write!(
                f,
                "The number of slaves you specified: {}, but found: {}",
                num, wc
            ),
            Self::NotResponding => write!(f, "One ore more slaves are not responding"),
            Self::FailedReadData => write!(f, "Failed to read data."),
            Self::CreateTimerError => write!(f, "Create Timer failed"),
            Self::DeleteTimerError => write!(f, "Delete Timer failed"),
        }
    }
}

impl error::Error for SOEMError {
    fn description(&self) -> &str {
        match *self {
            Self::NoSocketConnection(_) => "No socket connection",
            Self::SlaveNotFound(_, _) => "Mismatched numbers of slaves",
            Self::NotResponding => "One ore more slaves are not responding",
            Self::FailedReadData => "Failed to read data.",
            Self::CreateTimerError => "Create Timer failed",
            Self::DeleteTimerError => "Delete Timer failed",
        }
    }

    fn cause(&self) -> Option<&dyn error::Error> {
        match *self {
            Self::NoSocketConnection(_) => None,
            Self::SlaveNotFound(_, _) => None,
            Self::NotResponding => None,
            Self::FailedReadData => None,
            Self::CreateTimerError => None,
            Self::DeleteTimerError => None,
        }
    }
}
