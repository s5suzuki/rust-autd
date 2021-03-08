/*
 * File: soem_handler.rs
 * Project: src
 * Created Date: 30/08/2019
 * Author: Shun Suzuki
 * -----
 * Last Modified: 08/03/2021
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2019 Hapis Lab. All rights reserved.
 *
 */

use std::error::Error;

use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex, RwLock};
use std::thread::{self, JoinHandle};
use std::time::Instant;
use std::vec::Vec;

use autd_timer::Timer;
use libc::{c_char, c_void};

use crate::native_methods::*;
use crate::soem_error::SOEMError;

const EC_TIMEOUTSTATE: i32 = 2_000_000;
const EC_TIMEOUTRET: i32 = 2_000;
#[cfg(windows)]
const REALTIME_PRIORITY_CLASS: i32 = 0x0000_0100;

static SEND_COND: AtomicBool = AtomicBool::new(false);
static RTTHREAD_LOCK: AtomicBool = AtomicBool::new(false);

macro_rules! if_not_open_or_cannot_read {
    ($is_open:expr, $cnt:stmt) => {
        if let Ok(open) = $is_open.read() {
            if !*open {
                $cnt
            }
        }
    };
}

macro_rules! write_rwlock {
    ($x_rwlock:expr, $value: expr) => {
        if let Ok(mut x) = $x_rwlock.write() {
            *x = $value;
        }
    };
}

#[derive(Copy, Clone)]
pub struct ECConfig {
    pub header_size: usize,
    pub body_size: usize,
    pub input_frame_size: usize,
}

impl ECConfig {
    pub fn size(&self) -> usize {
        self.header_size + self.body_size + self.input_frame_size
    }
}

pub struct RuSOEM {
    timer_handle: Timer,
    is_open: Arc<RwLock<bool>>,
    ifname: std::ffi::CString,
    dev_num: u16,
    config: ECConfig,
    io_map: Arc<RwLock<Vec<u8>>>,
    cpy_handle: Option<JoinHandle<()>>,
    send_buf_q: Arc<(Mutex<VecDeque<Vec<u8>>>, Condvar)>,
    sync0_cyctime: u32,
}

#[allow(clippy::mutex_atomic)]
impl RuSOEM {
    pub fn new(ifname: &str, config: ECConfig) -> RuSOEM {
        RuSOEM {
            timer_handle: Timer::new(),
            is_open: Arc::new(RwLock::new(false)),
            config,
            dev_num: 0,
            ifname: std::ffi::CString::new(ifname.to_string()).unwrap(),
            io_map: Arc::new(RwLock::new(vec![])),
            cpy_handle: None,
            send_buf_q: Arc::new((Mutex::new(VecDeque::new()), Condvar::new())),
            sync0_cyctime: 0,
        }
    }

    pub fn start(
        &mut self,
        dev_num: u16,
        ec_sm2_cyctime_ns: u32,
        ec_sync0_cyctime_ns: u32,
    ) -> Result<(), Box<dyn Error>> {
        self.dev_num = dev_num;
        let size = self.config.size() * dev_num as usize;
        self.sync0_cyctime = ec_sync0_cyctime_ns;

        unsafe {
            #[cfg(windows)]
            {
                use winapi::um::processthreadsapi;
                let thread_handle = processthreadsapi::GetCurrentProcess();
                if processthreadsapi::SetThreadPriority(thread_handle, REALTIME_PRIORITY_CLASS) == 1
                {
                    eprintln!("Failed to SetPriorityClass");
                }
            }

            if ec_init(self.ifname.as_ptr() as *const c_char) != 1 {
                return Err(From::from(SOEMError::NoSocketConnection(
                    self.ifname.to_str().unwrap().to_string(),
                )));
            }

            self.io_map = Arc::new(RwLock::new(vec![0x00; size]));
            if let Ok(io_map) = self.io_map.read() {
                let wc = ec_config(0, io_map.as_ptr() as *const c_void) as u16;
                if wc != dev_num {
                    return Err(From::from(SOEMError::SlaveNotFound(wc, dev_num)));
                }
            };

            ec_configdc();
            ec_statecheck(0, EC_STATE_SAFE_OP, EC_TIMEOUTSTATE * 4);

            ec_slave[0].state = EC_STATE_OPERATIONAL;
            ec_send_processdata();
            ec_receive_processdata(EC_TIMEOUTRET);

            ec_writestate(0);

            let mut chk = 200;
            ec_statecheck(0, EC_STATE_OPERATIONAL, 50000);
            while chk > 0 && (ec_slave[0].state != EC_STATE_OPERATIONAL) {
                ec_statecheck(0, EC_STATE_OPERATIONAL, 50000);
                chk -= 1;
            }

            if ec_slave[0].state != EC_STATE_OPERATIONAL {
                return Err(SOEMError::NotResponding.into());
            }

            write_rwlock!(self.is_open, true);
            RuSOEM::setup_sync0(true, dev_num, self.sync0_cyctime);

            self.timer_handle.start(Self::rt_thread, ec_sm2_cyctime_ns);

            self.create_cpy_thread();
        }
        Ok(())
    }

    pub fn close(&mut self) {
        if_not_open_or_cannot_read!(self.is_open, return);

        write_rwlock!(self.is_open, false);
        let (send_lk, send_cvar) = &*self.send_buf_q;
        {
            let mut deq = send_lk.lock().unwrap();
            deq.clear();
        }
        send_cvar.notify_one();

        if let Some(jh) = self.cpy_handle.take() {
            jh.join().unwrap();
            self.cpy_handle = None;
        }

        if let Ok(mut io_map) = self.io_map.write() {
            let output_frame_size = self.config.header_size + self.config.body_size;
            unsafe {
                std::ptr::write_bytes(
                    io_map.as_mut_ptr(),
                    0x00,
                    self.dev_num as usize * output_frame_size,
                );
            }
        }

        self.timer_handle.close();

        unsafe {
            RuSOEM::setup_sync0(false, self.dev_num, self.sync0_cyctime);

            ec_slave[0].state = EC_STATE_INIT;
            ec_writestate(0);
            ec_statecheck(0, EC_STATE_INIT, EC_TIMEOUTSTATE);
            ec_close();
        }
    }

    pub fn send(&self, data: &[u8]) {
        let (send_lk, send_cvar) = &*self.send_buf_q;
        {
            let mut deq = send_lk.lock().unwrap();
            deq.push_back(data.to_vec());
        }
        send_cvar.notify_one();
    }

    pub fn is_open(&self) -> bool {
        match self.is_open.read() {
            Ok(is_open) => *is_open,
            _ => false,
        }
    }

    unsafe fn write_io_map(
        src: Vec<u8>,
        dst: *mut u8,
        dev_num: u16,
        header_size: usize,
        body_size: usize,
    ) {
        let size = src.len();
        let includes_body = (size - header_size) > 0;
        for i in 0..(dev_num as usize) {
            if includes_body {
                std::ptr::copy_nonoverlapping(
                    src.as_ptr().add(header_size + body_size * i),
                    dst.add((header_size + body_size) * i),
                    body_size,
                );
            }
            std::ptr::copy_nonoverlapping(
                src.as_ptr(),
                dst.add((header_size + body_size) * i + body_size),
                header_size,
            );
        }
    }

    pub fn read<T>(&self, data: &mut [T]) -> bool {
        if let Ok(io_map) = self.io_map.read() {
            Self::read_input(data, &io_map, self.config);
            true
        } else {
            false
        }
    }

    fn read_input<T>(data: &mut [T], io_map: &[u8], config: ECConfig) {
        let size = io_map.len();
        let output_frame_size = config.header_size + config.body_size;
        let dev_num = size / (output_frame_size + config.input_frame_size);
        let element_size = std::mem::size_of::<T>();
        let len = dev_num * config.input_frame_size / element_size;
        unsafe {
            std::ptr::copy_nonoverlapping(
                io_map.as_ptr().add(output_frame_size * dev_num) as *const T,
                data.as_mut_ptr(),
                len,
            );
        }
    }

    unsafe fn setup_sync0(activate: bool, dev_num: u16, cycle_time: u32) {
        let ref_time = Instant::now();
        for slave in 1..=dev_num {
            let elapsed = ref_time.elapsed().as_nanos();
            let shift = ((elapsed / cycle_time as u128) * cycle_time as u128) as i32;
            ec_dcsync0(slave, activate, cycle_time, -shift);
        }
    }

    unsafe fn create_cpy_thread(&mut self) {
        let is_open = self.is_open.clone();
        let send_buf_q = self.send_buf_q.clone();
        let dev_num = self.dev_num;
        let io_map = self.io_map.clone();
        let config = self.config;
        self.cpy_handle = Some(thread::spawn(move || {
            let (send_lk, send_cvar) = &*send_buf_q;
            let mut send_buf = send_lk.lock().unwrap();
            loop {
                if_not_open_or_cannot_read!(is_open, break);
                match send_buf.pop_front() {
                    None => send_buf = send_cvar.wait(send_buf).unwrap(),
                    Some(buf) => {
                        if let Ok(mut io_map) = io_map.write() {
                            RuSOEM::write_io_map(
                                buf,
                                io_map.as_mut_ptr(),
                                dev_num,
                                config.header_size,
                                config.body_size,
                            );
                        }
                        {
                            SEND_COND.store(false, Ordering::Release);
                            while !SEND_COND.load(Ordering::Acquire) {
                                if_not_open_or_cannot_read!(is_open, break);
                            }
                        }
                    }
                }
            }
        }));
    }

    #[inline]
    fn rt_thread() {
        unsafe {
            if let Ok(false) =
                RTTHREAD_LOCK.compare_exchange(false, true, Ordering::SeqCst, Ordering::Acquire)
            {
                let pre = SEND_COND.load(Ordering::Acquire);
                ec_send_processdata();
                if !pre {
                    SEND_COND.store(true, Ordering::Release);
                }
                ec_receive_processdata(EC_TIMEOUTRET);
                RTTHREAD_LOCK.store(false, Ordering::Release);
            }
        }
    }
}
