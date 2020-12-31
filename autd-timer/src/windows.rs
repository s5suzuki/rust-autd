/*
 * File: native_timer_wrapper.rs
 * Project: src
 * Created Date: 12/12/2019
 * Author: Shun Suzuki
 * -----
 * Last Modified: 31/12/2020
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2019 Hapis Lab. All rights reserved.
 *
 */

use winapi::shared::basetsd::DWORD_PTR;
use winapi::um::mmsystem::MMRESULT;
use winapi::um::threadpoollegacyapiset;
use winapi::um::timeapi::{timeBeginPeriod, timeEndPeriod};

type LPTIMECALLBACK = Option<
    unsafe extern "system" fn(
        u_timer_id: u32,
        u_msg: u32,
        dw_user: DWORD_PTR,
        dw1: DWORD_PTR,
        dw2: DWORD_PTR,
    ),
>;

const TIME_PERIODIC: u32 = 1;

#[link(name = "winmm")]
extern "C" {
    pub fn timeSetEvent(
        u_delay: u32,
        u_resolution: u32,
        lp_time_proc: LPTIMECALLBACK,
        dw_user: DWORD_PTR,
        fu_event: u32,
    ) -> MMRESULT;
    pub fn timeKillEvent(u_timer_id: u32) -> MMRESULT;
}

pub struct NativeTimerWrapper {
    timer_id: Option<u32>,
}

impl NativeTimerWrapper {
    pub fn new() -> NativeTimerWrapper {
        NativeTimerWrapper { timer_id: None }
    }

    pub fn start<P>(&mut self, cb: LPTIMECALLBACK, period_ns: u32, lp_param: *mut P) -> u32 {
        unsafe {
            let timer_queue = threadpoollegacyapiset::CreateTimerQueue();
            if timer_queue.is_null() {
                return 0;
            }

            let u_resolution = 1;
            timeBeginPeriod(u_resolution);
            let timer_id = timeSetEvent(
                period_ns / 1000 / 1000,
                u_resolution,
                cb,
                lp_param as usize,
                TIME_PERIODIC,
            );
            if timer_id != 0 {
                self.timer_id = Some(timer_id);
            }
            timer_id
        }
    }

    pub fn close(&mut self) -> i32 {
        if let Some(timer_id) = self.timer_id.take() {
            unsafe {
                timeEndPeriod(1);
                timeKillEvent(timer_id);
                0
            }
        } else {
            1
        }
    }
}

impl Drop for NativeTimerWrapper {
    fn drop(&mut self) {
        let _ = self.close();
    }
}
