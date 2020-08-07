/*
 * File: mod.rs
 * Project: controller
 * Created Date: 07/08/2020
 * Author: Shun Suzuki
 * -----
 * Last Modified: 07/08/2020
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2020 Hapis Lab. All rights reserved.
 *
 */

use std::error::Error;

use std::collections::VecDeque;
use std::mem::size_of;
use std::sync::{Arc, Condvar, Mutex, MutexGuard, RwLock};
use std::thread::{self, JoinHandle};

use crate::core::consts::*;
use crate::core::*;
use crate::geometry::Geometry;
use crate::link::Link;
use crate::modulation::Modulation;
use crate::sequence::PointSequence;

use autd_timer::Timer;

use crate::gain::{primitives::NullGain, Gain};

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
    /// # Arguments
    ///
    /// * `link` - Open device with a specific link.
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

    pub fn clear(&mut self) -> Result<bool, Box<dyn Error>> {
        let header = Self::make_header(CommandType::CmdClear);
        let dev_num = self.geometry().num_devices();
        self.send(header);
        let result = self.wait_msg_processed(dev_num, CommandType::CmdClear as u8, 0xFF, 200);
        Ok(result)
    }

    pub fn calibrate(&mut self) -> Result<bool, Box<dyn Error>> {
        let header = Self::make_header(CommandType::CmdInitRefClock);
        let dev_num = self.geometry().num_devices();
        self.send(header);
        let result =
            self.wait_msg_processed(dev_num, CommandType::CmdInitRefClock as u8, 0xFF, 5000);
        Ok(result)
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
            self.wait_msg_processed(dev_num, msg_id, 0xFF, 200);
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
            self.wait_msg_processed(dev_num, msg_id, 0xFF, 200);
        }
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

        let mut cpu_versions: Vec<u16> = vec![0x0000; size];
        let mut fpga_versions: Vec<u16> = vec![0x0000; size];

        let header = Self::make_header(CommandType::CmdReadCpuVerLsb);
        self.send(header);

        self.wait_msg_processed(size, CommandType::CmdReadCpuVerLsb as u8, 0xFF, 50);
        let rx_data = match &self.rx_data {
            Some(d) => d,
            None => return vec![],
        };

        for i in 0..size {
            cpu_versions[i] = rx_data[2 * i] as u16;
        }

        let header = Self::make_header(CommandType::CmdReadCpuVerMsb);
        self.send(header);

        self.wait_msg_processed(size, CommandType::CmdReadCpuVerMsb as u8, 0xFF, 50);
        let rx_data = match &self.rx_data {
            Some(d) => d,
            None => return vec![],
        };
        for i in 0..size {
            cpu_versions[i] |= (rx_data[2 * i] as u16) << 8;
        }

        let header = Self::make_header(CommandType::CmdReadFpgaVerLsb);
        self.send(header);

        self.wait_msg_processed(size, CommandType::CmdReadFpgaVerLsb as u8, 0xFF, 50);
        let rx_data = match &self.rx_data {
            Some(d) => d,
            None => return vec![],
        };
        for i in 0..size {
            fpga_versions[i] = rx_data[2 * i] as u16;
        }

        let header = Self::make_header(CommandType::CmdReadFpgaVerMsb);
        self.send(header);

        self.wait_msg_processed(size, CommandType::CmdReadFpgaVerMsb as u8, 0xFF, 50);
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
}

// Software STM
impl AUTD {
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
}

// Point Sequence
impl AUTD {
    pub fn append_sequence(&mut self, seq: PointSequence) {
        let mut seq = seq;
        let is_silent = self.is_silent();
        let dev_num = self.geometry().num_devices();
        while seq.sent() < seq.control_points().len() {
            let (msg_id, body) = AUTD::make_seq_body(&mut seq, &self.geometry(), is_silent);
            self.send(body);
            if seq.sent() == seq.control_points().len() {
                self.wait_msg_processed(dev_num, 0xC0, 0xE0, 200);
            } else {
                self.wait_msg_processed(dev_num, msg_id, 0xFF, 200);
            }
        }
        self.calibrate_seq();
    }

    fn calibrate_seq(&mut self) {
        let rx_data = match &self.rx_data {
            Some(d) => d,
            None => return,
        };
        let mut laps = Vec::with_capacity(rx_data.len() / 2);
        for j in 0..laps.capacity() {
            let lap_raw = ((rx_data[2 * j + 1] as u16) << 8) | rx_data[2 * j] as u16;
            laps.push(lap_raw & 0x03FF);
        }
        let minimum = laps.iter().min().unwrap();
        let diffs = laps.iter().map(|&d| d - minimum).collect::<Vec<_>>();
        let diff_max = *diffs.iter().max().unwrap();
        let diffs: Vec<u16> = if diff_max == 0 {
            return;
        } else if diff_max > 500 {
            let laps = laps
                .iter()
                .map(|&d| if d < 500 { d + 1000 } else { d })
                .collect::<Vec<_>>();
            let minimum = laps.iter().min().unwrap();
            laps.iter().map(|d| d - minimum).collect()
        } else {
            diffs
        };

        let dev_num = diffs.len();
        let calib_body = Self::make_calib_body(diffs);
        self.send(calib_body);
        self.wait_msg_processed(dev_num, 0xE0, 0xE0, 200);
    }

    fn make_calib_body(diffs: Vec<u16>) -> Vec<u8> {
        let header = RxGlobalHeader::new_with_cmd(CommandType::CmdCalibSeqClock);
        let mut body =
            vec![0x00; size_of::<RxGlobalHeader>() + NUM_TRANS_IN_UNIT * 2 * diffs.len()];
        unsafe {
            std::ptr::copy_nonoverlapping(
                &header as *const RxGlobalHeader as *const u8,
                body.as_mut_ptr(),
                size_of::<RxGlobalHeader>(),
            );
            let mut cursor = size_of::<RxGlobalHeader>();
            for diff in diffs {
                body[cursor] = (diff & 0x00FF) as u8;
                body[cursor + 1] = ((diff & 0xFF00) >> 8) as u8;
                cursor += NUM_TRANS_IN_UNIT * 2;
            }
        }
        body
    }

    fn make_seq_body(
        seq: &mut PointSequence,
        geometry: &Geometry,
        is_silent: bool,
    ) -> (u8, Vec<u8>) {
        let num_devices = geometry.num_devices();
        let size = size_of::<RxGlobalHeader>() + NUM_TRANS_IN_UNIT * 2 * num_devices;

        let mut body = vec![0x00; size];
        let send_size = num::clamp(seq.control_points().len() - seq.sent(), 0, 40);

        let mut ctrl_flags = RxGlobalControlFlags::SEQ_MODE;
        if is_silent {
            ctrl_flags |= RxGlobalControlFlags::SILENT;
        }

        if seq.sent() == 0 {
            ctrl_flags |= RxGlobalControlFlags::SEQ_BEGIN;
        }
        if seq.sent() + send_size >= seq.control_points().len() {
            ctrl_flags |= RxGlobalControlFlags::SEQ_END;
        }
        let msg_id = unsafe {
            let header =
                RxGlobalHeader::new_seq(ctrl_flags, send_size as u16, seq.sampling_freq_div());
            let src_ptr = &header as *const RxGlobalHeader as *const u8;
            let dst_ptr = body.as_mut_ptr();
            std::ptr::copy_nonoverlapping(src_ptr, dst_ptr, size_of::<RxGlobalHeader>());
            header.msg_id
        };

        let mut cursor = size_of::<RxGlobalHeader>();
        unsafe {
            const FIXED_NUM_UNIT: f64 = ULTRASOUND_WAVELENGTH / 256.0;
            for device in 0..num_devices {
                let mut foci = Vec::with_capacity(send_size as usize * 10);
                for i in 0..(send_size as usize) {
                    let v64 = geometry.local_position(device, seq.control_points()[seq.sent() + i]);
                    let x = (v64.x / FIXED_NUM_UNIT) as i32 as u32;
                    let y = (v64.y / FIXED_NUM_UNIT) as i32 as u32;
                    let z = (v64.z / FIXED_NUM_UNIT) as i32 as u32;
                    foci.push((x & 0x000000FF) as u8);
                    foci.push(((x & 0x0000FF00) >> 8) as u8);
                    foci.push((((x & 0x80000000) >> 24) | ((x & 0x007F0000) >> 16)) as u8);
                    foci.push((y & 0x000000FF) as u8);
                    foci.push(((y & 0x0000FF00) >> 8) as u8);
                    foci.push((((y & 0x80000000) >> 24) | ((y & 0x007F0000) >> 16)) as u8);
                    foci.push((z & 0x000000FF) as u8);
                    foci.push(((z & 0x0000FF00) >> 8) as u8);
                    foci.push((((z & 0x80000000) >> 24) | ((z & 0x007F0000) >> 16)) as u8);
                    foci.push(0xFF); // amp
                }
                let src_ptr = foci.as_ptr() as *const u8;
                let dst_ptr = body.as_mut_ptr().add(cursor);

                std::ptr::copy_nonoverlapping(src_ptr, dst_ptr, foci.len());
                cursor += NUM_TRANS_IN_UNIT * 2;
            }
        }
        seq.send(send_size);
        (msg_id, body)
    }
}

// Private functions
impl AUTD {
    fn make_header(command: CommandType) -> Vec<u8> {
        let size = std::mem::size_of::<RxGlobalHeader>();
        let mut header_bytes = vec![0x00; size];
        let header = RxGlobalHeader::new_with_cmd(command);
        unsafe {
            std::ptr::copy_nonoverlapping(
                &header as *const RxGlobalHeader as *const u8,
                header_bytes.as_mut_ptr(),
                size,
            )
        }
        header_bytes
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
            let header = RxGlobalHeader::new_op(ctrl_flags, mod_data);
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

    fn wait_msg_processed(
        &mut self,
        dev_num: usize,
        msg_id: u8,
        mask: u8,
        max_trial: usize,
    ) -> bool {
        let link = match &self.link {
            Some(link) => link.clone(),
            None => return false,
        };

        let buffer_len = dev_num * INPUT_FRAME_SIZE;

        for _ in 0..max_trial {
            let rx_data = {
                let mut l = (&*link).lock().unwrap();
                l.read(buffer_len as u32)
            };

            let rx_data = match rx_data {
                Ok(data) => data,
                Err(_) => return false,
            };

            let processed = (0..dev_num)
                .map(|dev| rx_data[dev as usize * INPUT_FRAME_SIZE + 1])
                .filter(|&proc_id| (proc_id & mask) == msg_id)
                .count();
            self.rx_data = Some(rx_data);

            if processed == dev_num {
                return true;
            }

            let wait_t = (EC_TRAFFIC_DELAY * 1000.0 / EC_DEVICE_PER_FRAME as f64 * dev_num as f64)
                .ceil() as u64;
            let wait_t = 1.max(wait_t);
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
