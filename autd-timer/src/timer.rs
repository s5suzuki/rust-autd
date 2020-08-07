/*
 * File: timer.rs
 * Project: src
 * Created Date: 23/05/2020
 * Author: Shun Suzuki
 * -----
 * Last Modified: 24/05/2020
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2020 Hapis Lab. All rights reserved.
 *
 */

use crate::NativeTimerWrapper;

#[cfg(target_os = "windows")]
use winapi::um::winnt::PVOID;

#[cfg(target_os = "linux")]
use libc::{c_int, c_void, siginfo_t};

#[cfg(target_os = "macos")]
use libc::c_void;

pub struct Timer {
    native_timer: NativeTimerWrapper,
    cb: Option<Box<dyn FnMut() + Send>>,
}

impl Timer {
    pub fn new() -> Self {
        Self {
            native_timer: NativeTimerWrapper::new(),
            cb: None,
        }
    }

    pub fn start<F>(&mut self, cb: F, period_ns: u32)
    where
        F: 'static + FnMut() + Send,
    {
        self.cb = Some(Box::new(cb));
        let ptr = self as *mut Self;
        self.native_timer
            .start(Some(Self::rt_thread), period_ns, ptr);
    }

    pub fn close(&mut self) {
        self.native_timer.close();
    }

    #[cfg(target_os = "windows")]
    unsafe extern "system" fn rt_thread(lp_param: PVOID, _t: u8) {
        let ptr = lp_param as *mut Self;
        if let Some(cb) = &mut (*ptr).cb {
            cb();
        }
    }

    #[cfg(target_os = "linux")]
    unsafe extern "C" fn rt_thread(_sig: c_int, si: *mut siginfo_t, _uc: *mut c_void) {
        let ptr = Self::get_ptr(si);
        let ptr = ptr as *mut Self;
        if let Some(cb) = &mut (*ptr).cb {
            cb();
        }
    }

    #[cfg(target_os = "linux")]
    #[allow(deprecated)]
    unsafe extern "C" fn get_ptr(si: *mut siginfo_t) -> u64{
        // TODO: This depends on the deprecated field of libc crate, and may only work on a specific platforms.
        let ptr_lsb = (*si)._pad[3];
        let ptr_msb = (*si)._pad[4];
        ((ptr_msb as u64) << 32) | (ptr_lsb as u64 & 0xFFFF_FFFF)
    }

    #[cfg(target_os = "macos")]
    unsafe extern "C" fn rt_thread(ptr: *const c_void) {
        let ptr = ptr as *mut Self;
        if let Some(cb) = &mut (*ptr).cb {
            cb();
        }
    }
}

impl Default for Timer {
    fn default() -> Self {
        Self::new()
    }
}
