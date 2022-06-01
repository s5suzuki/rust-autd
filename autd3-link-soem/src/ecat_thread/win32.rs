/*
 * File: win32.rs
 * Project: ecat_thread
 * Created Date: 03/05/2022
 * Author: Shun Suzuki
 * -----
 * Last Modified: 31/05/2022
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2022 Shun Suzuki. All rights reserved.
 *
 */

use std::{
    marker::PhantomData,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use autd3_core::{RxDatagram, TxDatagram};
use crossbeam_channel::{Receiver, Sender};
use libc::timespec;
use once_cell::sync::Lazy;

use windows::Win32::{
    Media::*,
    Networking::WinSock::timeval,
    System::Performance::{QueryPerformanceCounter, QueryPerformanceFrequency},
    System::Threading::*,
};

use crate::{iomap::IOMap, native_methods::*};

use super::{
    error_handler::EcatErrorHandler,
    mode::{HighPrecision, Normal},
    utils::*,
};

static PERFORMANCE_FREQUENCY: Lazy<i64> = Lazy::new(|| unsafe {
    let mut freq = 0;
    QueryPerformanceFrequency(&mut freq as *mut _);
    freq
});

fn nanosleep(t: i64) {
    unsafe {
        let mut start = 0;
        QueryPerformanceCounter(&mut start as *mut _);

        let pf = *PERFORMANCE_FREQUENCY;
        let sleep = t * pf / (1000 * 1000 * 1000);
        loop {
            let mut now = 0;
            QueryPerformanceCounter(&mut now as *mut _);
            if now - start > sleep {
                break;
            }
        }
    }
}

fn add_timespec(ts: &mut timespec, addtime: i64) {
    let nsec = addtime % 1000000000;
    let sec = (addtime - nsec) / 1000000000;
    ts.tv_sec += sec;
    ts.tv_nsec += nsec as i32;
    if ts.tv_nsec >= 1000000000 {
        let nsec = ts.tv_nsec % 1000000000;
        ts.tv_sec += ((ts.tv_nsec - nsec) / 1000000000) as i64;
        ts.tv_nsec = nsec;
    }
}

fn timed_wait(abs_time: &timespec) {
    let mut tp = timeval {
        tv_sec: 0,
        tv_usec: 0,
    };
    unsafe {
        osal_gettimeofday(&mut tp as *mut _ as *mut _, std::ptr::null_mut() as *mut _);
    }

    let sleep = (abs_time.tv_sec - tp.tv_sec as i64) * 1000000000
        + (abs_time.tv_nsec - tp.tv_usec * 1000) as i64;

    if sleep > 0 {
        std::thread::sleep(std::time::Duration::from_nanos(sleep as _));
    }
}

fn timed_wait_h(abs_time: &timespec) {
    let mut tp = timeval {
        tv_sec: 0,
        tv_usec: 0,
    };
    unsafe {
        osal_gettimeofday(&mut tp as *mut _ as *mut _, std::ptr::null_mut() as *mut _);
    }

    let sleep = (abs_time.tv_sec - tp.tv_sec as i64) * 1000000000
        + (abs_time.tv_nsec - tp.tv_usec * 1000) as i64;

    if sleep > 0 {
        nanosleep(sleep);
    }
}

pub struct EcatThreadHandler<F: Fn(&str), M> {
    io_map: Box<IOMap>,
    is_running: Arc<AtomicBool>,
    receiver: Receiver<TxDatagram>,
    sender: Sender<RxDatagram>,
    expected_wkc: i32,
    cycletime: i64,
    error_handler: EcatErrorHandler<F>,
    _phantom_data: PhantomData<M>,
}

impl<F: Fn(&str) + Send, M> EcatThreadHandler<F, M> {
    pub fn new(
        io_map: Box<IOMap>,
        is_running: Arc<AtomicBool>,
        receiver: Receiver<TxDatagram>,
        sender: Sender<RxDatagram>,
        expected_wkc: i32,
        cycletime: i64,
        error_handler: EcatErrorHandler<F>,
    ) -> Self {
        Self {
            io_map,
            is_running,
            receiver,
            sender,
            expected_wkc,
            cycletime,
            error_handler,
            _phantom_data: PhantomData,
        }
    }
}

impl<F: Fn(&str) + Send> EcatThreadHandler<F, Normal> {
    pub fn run(&mut self) {
        unsafe {
            let u_resolution = 1;
            timeBeginPeriod(u_resolution);

            let h_process = GetCurrentProcess();
            let priority = GetPriorityClass(h_process);
            SetPriorityClass(h_process, REALTIME_PRIORITY_CLASS);

            let mut ts = timespec {
                tv_sec: 0,
                tv_nsec: 0,
            };

            let mut tp = timeval {
                tv_sec: 0,
                tv_usec: 0,
            };
            osal_gettimeofday(&mut tp as *mut _ as *mut _, std::ptr::null_mut() as *mut _);

            let cyctime_us = (self.cycletime / 1000) as i32;

            ts.tv_sec = tp.tv_sec as _;
            let ht = ((tp.tv_usec / cyctime_us) + 1) * cyctime_us;
            ts.tv_nsec = ht * 1000;

            let mut toff = 0;
            while self.is_running.load(Ordering::Acquire) {
                add_timespec(&mut ts, self.cycletime + toff);

                timed_wait(&ts);

                if ec_slave[0].state == ec_state_EC_STATE_SAFE_OP as _ {
                    ec_slave[0].state = ec_state_EC_STATE_OPERATIONAL as _;
                    ec_writestate(0);
                }

                if let Ok(tx) = self.receiver.try_recv() {
                    self.io_map.copy_from(tx);
                }

                ec_send_processdata();
                if ec_receive_processdata(EC_TIMEOUTRET as i32) != self.expected_wkc
                    && !self.error_handler.handle()
                {
                    return;
                }

                self.sender.send(self.io_map.input()).unwrap();

                ec_sync(ec_DCtime, self.cycletime, &mut toff);
            }

            timeEndPeriod(1);
            SetPriorityClass(h_process, PROCESS_CREATION_FLAGS(priority));
        }
    }
}

impl<F: Fn(&str) + Send> EcatThreadHandler<F, HighPrecision> {
    pub fn run(&mut self) {
        unsafe {
            let mut ts = timespec {
                tv_sec: 0,
                tv_nsec: 0,
            };

            let mut tp = timeval {
                tv_sec: 0,
                tv_usec: 0,
            };
            osal_gettimeofday(&mut tp as *mut _ as *mut _, std::ptr::null_mut() as *mut _);

            let cyctime_us = (self.cycletime / 1000) as i32;

            ts.tv_sec = tp.tv_sec as _;
            let ht = ((tp.tv_usec / cyctime_us) + 1) * cyctime_us;
            ts.tv_nsec = ht * 1000;

            let mut toff = 0;
            while self.is_running.load(Ordering::Acquire) {
                add_timespec(&mut ts, self.cycletime + toff);

                timed_wait_h(&ts);

                if ec_slave[0].state == ec_state_EC_STATE_SAFE_OP as _ {
                    ec_slave[0].state = ec_state_EC_STATE_OPERATIONAL as _;
                    ec_writestate(0);
                }

                if let Ok(tx) = self.receiver.try_recv() {
                    self.io_map.copy_from(tx);
                }

                ec_send_processdata();
                if ec_receive_processdata(EC_TIMEOUTRET as i32) != self.expected_wkc
                    && !self.error_handler.handle()
                {
                    return;
                }

                self.sender.send(self.io_map.input()).unwrap();

                ec_sync(ec_DCtime, self.cycletime, &mut toff);
            }
        }
    }
}
