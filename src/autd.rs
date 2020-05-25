/*
 * File: controller.rs
 * Project: autd
 * Created Date: 02/09/2019
 * Author: Shun Suzuki
 * -----
 * Last Modified: 25/05/2020
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2019 Hapis Lab. All rights reserved.
 *
 */

use std::error::Error;

use std::collections::VecDeque;
use std::mem::size_of;
use std::sync::{Arc, Condvar, Mutex, MutexGuard, RwLock};
use std::thread::{self, JoinHandle};

use autd_core::consts::*;
use autd_core::*;
use autd_gain::Gain;
use autd_geometry::Geometry;
use autd_link::Link;
use autd_modulation::Modulation;
use autd_timer::Timer;

use crate::prelude::*;

type GainPtr = Box<dyn Gain>;
type GainQueue = VecDeque<GainPtr>;
type ModulationQueue = VecDeque<Modulation>;

struct SendQueue {
    gain_q: GainQueue,
    modulation_q: ModulationQueue,
}

/// The structure that controls AUTDs.
#[repr(C)]
pub struct AUTD {
    geometry: Arc<Mutex<Geometry>>,
    is_open: Arc<RwLock<bool>>,
    is_silent: Arc<RwLock<bool>>,
    link: Option<Arc<Mutex<Box<dyn Link>>>>,
    build_gain_q: Arc<(Mutex<GainQueue>, Condvar)>,
    send_gain_q: Arc<(Mutex<SendQueue>, Condvar)>,
    build_th_handle: Option<JoinHandle<()>>,
    send_th_handle: Option<JoinHandle<()>>,
    stm_gains: Arc<Mutex<Vec<GainPtr>>>,
    stm_timer: Timer,
    rx_data: Option<Vec<u8>>,
}

impl AUTD {
    /// constructor
    pub fn create() -> AUTD {
        let send_gain_q = Arc::new((
            Mutex::new(SendQueue {
                gain_q: GainQueue::new(),
                modulation_q: ModulationQueue::new(),
            }),
            Condvar::new(),
        ));
        AUTD {
            link: None,
            is_open: Arc::new(RwLock::new(true)),
            is_silent: Arc::new(RwLock::new(true)),
            geometry: Arc::new(Mutex::new(Default::default())),
            build_gain_q: Arc::new((Mutex::new(VecDeque::new()), Condvar::new())),
            send_gain_q,
            build_th_handle: None,
            send_th_handle: None,
            stm_gains: Arc::new(Mutex::new(Vec::new())),
            stm_timer: Timer::new(),
            rx_data: None,
        }
    }

    /// Open AUTDs
    ///
    /// You must call [add_device](#method.add_device) or [add_device_quaternion](#method.add_device_quaternion) before.
    ///
    /// # Arguments
    ///
    /// * `ifname` - A string slice that holds the name of the interface name connected to AUTDs.
    ///             With SOEM, you can get ifname via [EthernetAdapters](../struct.EthernetAdapters.html).
    /// * `link_type` - Only SOEM is supported.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use autd3::AUTD;
    /// use autd3::utils::Vector3;
    /// use autd3::LinkType;
    ///
    /// let mut autd = AUTD::create();
    ///
    /// autd.geometry().add_device(Vector3::zeros(), Vector3::zeros());
    ///
    /// let ifname = "interface name";
    /// match autd.open(ifname, LinkType::SOEM) {
    ///     Ok(()) => (),
    ///     Err(e) => println!("{}", e),
    /// }
    /// ```
    pub fn open<L: Link + 'static>(&mut self, link: L) -> Result<(), Box<dyn Error>> {
        let mut link = Box::new(link);
        link.open()?;
        self.link = Some(Arc::new(Mutex::new(link)));
        self.init_pipeline();
        Ok(())
    }

    pub fn geometry(&self) -> MutexGuard<'_, Geometry> {
        self.geometry.lock().unwrap()
    }

    pub fn set_silent_mode(&mut self, silent: bool) {
        if let Ok(mut is_silent) = self.is_silent.write() {
            *is_silent = silent;
        };
    }

    pub fn calibrate(&mut self) -> Result<bool, Box<dyn Error>> {
        let link = match &self.link {
            Some(link) => link.clone(),
            None => return Ok(false),
        };
        if self.geometry().num_devices() == 1 || self.geometry().num_devices() == 0 {
            return Ok(true);
        }
        let mut l = (&*link).lock().unwrap();
        l.calibrate()
    }

    pub fn close(mut self) {
        self.close_impl();
    }

    pub fn is_open(&self) -> bool {
        if let Ok(open) = self.is_open.read() {
            *open
        } else {
            false
        }
    }

    pub fn is_silent(&self) -> bool {
        if let Ok(is_silent) = self.is_silent.read() {
            *is_silent
        } else {
            true
        }
    }

    pub fn remaining_in_buffer(&self) -> usize {
        let (build_lk, _) = &*self.build_gain_q;
        let remain_build = {
            let build_q = build_lk.lock().unwrap();
            build_q.len()
        };
        let (send_lk, _) = &*self.send_gain_q;
        let remain_send = {
            let send_q = send_lk.lock().unwrap();
            send_q.gain_q.len() + send_q.modulation_q.len()
        };
        remain_build + remain_send
    }

    pub fn stop(&mut self) {
        self.finish_stm();
        self.append_gain_sync(NullGain::create());
    }

    pub fn append_gain(&mut self, gain: GainPtr) {
        let (build_lk, build_cvar) = &*self.build_gain_q;
        {
            let mut build_q = build_lk.lock().unwrap();
            build_q.push_back(gain);
        }
        build_cvar.notify_one();
    }

    pub fn append_gain_sync(&mut self, gain: GainPtr) {
        self.append_gain_sync_with_wait(gain, false)
    }

    pub fn append_gain_sync_with_wait(&mut self, mut gain: GainPtr, wait_for_send: bool) {
        {
            let geo = self.geometry();
            gain.build(&geo);
        }
        let dev_num = self.geometry().num_devices();
        let is_silent = self.is_silent();
        let (msg_id, body) = AUTD::make_body(Some(gain), None, dev_num, is_silent);
        self.send(body);
        if wait_for_send {
            self.wait_msg_processed(msg_id, 200);
        }
    }

    pub fn append_modulation(&mut self, modulation: Modulation) {
        let (send_lk, send_cvar) = &*self.send_gain_q;
        {
            let mut deq = send_lk.lock().unwrap();
            deq.modulation_q.push_back(modulation);
        }
        send_cvar.notify_one();
    }

    pub fn append_modulation_sync(&mut self, modulation: Modulation) {
        let mut modulation = modulation;
        let dev_num = self.geometry().num_devices();
        while modulation.sent() < modulation.buffer().len() {
            let is_silent = self.is_silent();
            let (msg_id, body) = AUTD::make_body(None, Some(&mut modulation), dev_num, is_silent);
            self.send(body);
            self.wait_msg_processed(msg_id, 200);
        }
    }

    pub fn append_stm_gains(&mut self, gains: Vec<GainPtr>) {
        self.stop_stm();
        let mut stm_gains = self.stm_gains.lock().unwrap();
        stm_gains.extend(gains);
    }

    pub fn start_stm(&mut self, freq: f64) {
        let len = { self.stm_gains.lock().unwrap().len() };
        assert!(len != 0);
        let itvl_ms = 1000. / freq / len as f64;

        let geometry = self.geometry.lock().unwrap();
        let dev_num = geometry.num_devices();
        let is_silent = self.is_silent();
        let mut stm_gains = self.stm_gains.lock().unwrap();
        let mut body_q = Vec::<Vec<u8>>::new();
        for _ in 0..stm_gains.len() {
            if let Some(mut gain) = stm_gains.pop() {
                gain.build(&geometry);
                let (_, body) = AUTD::make_body(Some(gain), None, dev_num, is_silent);
                body_q.push(body);
            }
        }

        let link = match &self.link {
            Some(link) => link.clone(),
            None => return,
        };
        let is_open = self.is_open.clone();
        let mut idx = 0;
        self.stm_timer.start(
            move || {
                let body = &body_q[idx % len];
                let mut body_copy = Vec::with_capacity(body.len());
                unsafe {
                    body_copy.set_len(body.len());
                    std::ptr::copy_nonoverlapping(
                        body.as_ptr(),
                        body_copy.as_mut_ptr(),
                        body.len(),
                    );
                }
                Self::send_impl(link.clone(), is_open.clone(), body_copy);
                idx = (idx + 1) % len;
            },
            (itvl_ms * 1000. * 1000.) as u32,
        );
    }

    pub fn stop_stm(&mut self) {
        self.stm_timer.close();
    }

    pub fn finish_stm(&mut self) {
        self.stop_stm();
        let mut stm_gains = self.stm_gains.lock().unwrap();
        stm_gains.clear();
    }

    pub fn flush(&mut self) {
        let (build_lk, _) = &*self.build_gain_q;
        {
            let mut build_q = build_lk.lock().unwrap();
            build_q.clear();
        }
        let (send_lk, _) = &*self.send_gain_q;
        {
            let mut send_q = send_lk.lock().unwrap();
            send_q.gain_q.clear();
            send_q.modulation_q.clear();
        }
    }

    pub fn firmware_info_list(&mut self) -> Vec<FirmwareInfo> {
        let size = self.geometry().num_devices();

        let make_header = |command: CommandType| {
            let size = std::mem::size_of::<RxGlobalHeader>();
            let mut header_bytes = vec![0x00; size];
            unsafe {
                let header = header_bytes.as_mut_ptr() as *mut RxGlobalHeader;
                (*header).msg_id = command as u8;
                (*header).command = command;
                header_bytes
            }
        };

        let mut cpu_versions: Vec<u16> = vec![0x0000; size];
        let mut fpga_versions: Vec<u16> = vec![0x0000; size];

        let header = make_header(CommandType::CmdReadCpuVerLsb);
        self.send(header);

        self.wait_msg_processed(CommandType::CmdReadCpuVerLsb as u8, 50);
        let rx_data = match &self.rx_data {
            Some(d) => d,
            None => return vec![],
        };

        for i in 0..size {
            cpu_versions[i] = rx_data[2 * i] as u16;
        }

        let header = make_header(CommandType::CmdReadCpuVerMsb);
        self.send(header);

        self.wait_msg_processed(CommandType::CmdReadCpuVerMsb as u8, 50);
        let rx_data = match &self.rx_data {
            Some(d) => d,
            None => return vec![],
        };
        for i in 0..size {
            cpu_versions[i] |= (rx_data[2 * i] as u16) << 8;
        }

        let header = make_header(CommandType::CmdReadFpgaVerLsb);
        self.send(header);

        self.wait_msg_processed(CommandType::CmdReadFpgaVerLsb as u8, 50);
        let rx_data = match &self.rx_data {
            Some(d) => d,
            None => return vec![],
        };
        for i in 0..size {
            fpga_versions[i] = rx_data[2 * i] as u16;
        }

        let header = make_header(CommandType::CmdReadFpgaVerMsb);
        self.send(header);

        self.wait_msg_processed(CommandType::CmdReadFpgaVerMsb as u8, 50);
        let rx_data = match &self.rx_data {
            Some(d) => d,
            None => return vec![],
        };
        for i in 0..size {
            fpga_versions[i] |= (rx_data[2 * i] as u16) << 8;
        }

        let mut res = Vec::with_capacity(size);
        for i in 0..size {
            let firm_info = FirmwareInfo::new(i as u16, cpu_versions[i], fpga_versions[i]);
            res.push(firm_info);
        }

        res
    }

    fn send(&mut self, data: Vec<u8>) {
        let link = match &self.link {
            Some(link) => link.clone(),
            None => return,
        };
        let is_open = self.is_open.clone();
        Self::send_impl(link, is_open, data);
    }

    fn send_impl(link: Arc<Mutex<Box<dyn Link>>>, is_open: Arc<RwLock<bool>>, data: Vec<u8>) {
        let mut l = (&*link).lock().unwrap();
        l.send(data).unwrap_or_else(|err| {
            eprintln!("{}", err);
            l.close().unwrap_or_else(|err| eprintln!("{}", err));
            if let Ok(mut open) = is_open.write() {
                *open = false;
            }
        });
    }

    fn init_pipeline(&mut self) {
        // Build thread
        let geometry = self.geometry.clone();
        let build_gain_q = self.build_gain_q.clone();
        let send_gain_q = self.send_gain_q.clone();
        let is_open = self.is_open.clone();
        self.build_th_handle = Some(thread::spawn(move || {
            let (build_lk, build_cvar) = &*build_gain_q;
            loop {
                if let Ok(open) = is_open.read() {
                    if !*open {
                        break;
                    }
                }
                let mut gain_q = build_lk.lock().unwrap();
                let gain = match gain_q.pop_front() {
                    None => {
                        let _q = build_cvar.wait(gain_q).unwrap();
                        continue;
                    }
                    Some(mut gain) => {
                        let geo = geometry.lock().unwrap();
                        gain.build(&geo);
                        gain
                    }
                };

                let (send_lk, send_cvar) = &*send_gain_q;
                {
                    let mut deq = send_lk.lock().unwrap();
                    deq.gain_q.push_back(gain);
                }
                send_cvar.notify_all();
            }
        }));

        // Send thread
        let link = match &self.link {
            Some(link) => link.clone(),
            None => return,
        };
        let send_gain_q = self.send_gain_q.clone();
        let geometry = self.geometry.clone();
        let is_open = self.is_open.clone();
        let is_silent = self.is_silent.clone();
        self.send_th_handle = Some(thread::spawn(move || {
            let (send_lk, send_cvar) = &*send_gain_q;
            loop {
                if let Ok(open) = is_open.read() {
                    if !*open {
                        break;
                    }
                }
                let mut send_buf = send_lk.lock().unwrap();
                match (
                    send_buf.gain_q.pop_front(),
                    send_buf.modulation_q.get_mut(0),
                ) {
                    (None, None) => {
                        let _q = send_cvar.wait(send_buf).unwrap();
                    }
                    (Some(g), None) => {
                        let dev_num = geometry.lock().unwrap().num_devices();
                        let is_silent = match is_silent.read() {
                            Ok(is_silent) => *is_silent,
                            Err(_) => true,
                        };
                        let (_, body) = AUTD::make_body(Some(g), None, dev_num, is_silent);
                        Self::send_impl(link.clone(), is_open.clone(), body);
                    }
                    (g, Some(m)) => {
                        let dev_num = geometry.lock().unwrap().num_devices();
                        let is_silent = match is_silent.read() {
                            Ok(is_silent) => *is_silent,
                            Err(_) => true,
                        };
                        let (_, body) = AUTD::make_body(g, Some(m), dev_num, is_silent);
                        Self::send_impl(link.clone(), is_open.clone(), body);
                        if m.buffer().len() <= m.sent() {
                            send_buf.modulation_q.pop_front();
                        }
                    }
                }
            }
        }));
    }

    fn make_body(
        gain: Option<GainPtr>,
        modulation: Option<&mut Modulation>,
        num_devices: usize,
        is_silent: bool,
    ) -> (u8, Vec<u8>) {
        let num_bodies = if gain.is_some() { num_devices } else { 0 };
        let size = size_of::<RxGlobalHeader>() + NUM_TRANS_IN_UNIT * 2 * num_bodies;

        let mut body = vec![0x00; size];
        let mut ctrl_flags = RxGlobalControlFlags::NONE;
        if is_silent {
            ctrl_flags |= RxGlobalControlFlags::SILENT;
        }
        let mut mod_data: &[u8] = &[];
        match modulation {
            None => (),
            Some(modulation) => {
                let sent = modulation.sent();
                let mod_size = num::clamp(modulation.buffer().len() - sent, 0, MOD_FRAME_SIZE);
                if sent == 0 {
                    ctrl_flags |= RxGlobalControlFlags::LOOP_BEGIN;
                }
                if sent + mod_size >= modulation.buffer().len() {
                    ctrl_flags |= RxGlobalControlFlags::LOOP_END;
                }
                modulation.send(mod_size);
                mod_data = &modulation.buffer()[sent..(sent + mod_size)];
            }
        }
        let msg_id = unsafe {
            let header = RxGlobalHeader::new(ctrl_flags, mod_data);
            let src_ptr = &header as *const RxGlobalHeader as *const u8;
            let dst_ptr = body.as_mut_ptr();
            std::ptr::copy_nonoverlapping(src_ptr, dst_ptr, size_of::<RxGlobalHeader>());
            header.msg_id
        };

        match gain {
            None => (),
            Some(gain) => {
                let mut cursor = size_of::<RxGlobalHeader>();
                let byte_size = NUM_TRANS_IN_UNIT * 2;
                let gain_ptr = gain.get_data().as_ptr();
                unsafe {
                    for i in 0..num_devices {
                        let src_ptr = gain_ptr.add(i * byte_size);
                        let dst_ptr = body.as_mut_ptr().add(cursor);

                        std::ptr::copy_nonoverlapping(src_ptr, dst_ptr, byte_size);
                        cursor += byte_size;
                    }
                }
            }
        }
        (msg_id, body)
    }

    fn wait_msg_processed(&mut self, msg_id: u8, max_trial: usize) -> bool {
        let link = match &self.link {
            Some(link) => link.clone(),
            None => return false,
        };

        let num_dev = self.geometry().num_devices();
        let buffer_len = num_dev * INPUT_FRAME_SIZE;

        for _ in 0..max_trial {
            let rx_data = {
                let mut l = (&*link).lock().unwrap();
                l.read(buffer_len as u32)
            };

            let rx_data = match rx_data {
                Ok(data) => data,
                Err(_) => return false,
            };

            let processed = (0..num_dev)
                .map(|dev| rx_data[dev as usize * INPUT_FRAME_SIZE + 1])
                .filter(|&proc_id| proc_id == msg_id)
                .count();
            self.rx_data = Some(rx_data);

            if processed == num_dev {
                return true;
            }

            let wait_t = (EC_TRAFFIC_DELAY * 1000.0 / EC_DEVICE_PER_FRAME as f64 * num_dev as f64)
                .ceil() as u64;
            std::thread::sleep(std::time::Duration::from_millis(wait_t));
        }
        false
    }

    fn close_impl(&mut self) {
        if let Ok(open) = self.is_open.read() {
            if !*open {
                return;
            }
        }

        self.finish_stm();
        self.flush();
        self.append_gain_sync_with_wait(NullGain::create(), true);

        if let Ok(mut open) = self.is_open.write() {
            *open = false;
        }

        if let Some(jh) = self.build_th_handle.take() {
            let (_, build_cvar) = &*self.build_gain_q;
            build_cvar.notify_one();
            jh.join().unwrap();
        }

        if let Some(jh) = self.send_th_handle.take() {
            let (_, send_cvar) = &*self.send_gain_q;
            send_cvar.notify_one();
            jh.join().unwrap();
        }

        match &self.link {
            Some(link) => (&*link)
                .lock()
                .unwrap()
                .close()
                .unwrap_or_else(|err| eprintln!("{}", err)),
            None => (),
        };
    }
}

impl Drop for AUTD {
    fn drop(&mut self) {
        self.close_impl();
    }
}
