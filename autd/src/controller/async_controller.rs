/*
 * File: async_controller.rs
 * Project: controller
 * Created Date: 30/12/2020
 * Author: Shun Suzuki
 * -----
 * Last Modified: 30/12/2020
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2020 Hapis Lab. All rights reserved.
 *
 */

use std::{
    collections::VecDeque,
    sync::{Arc, Condvar, Mutex, RwLock},
    thread::{self, JoinHandle},
};

use super::{autd_logic::AUTDLogic, GainPtr, ModPtr};
use crate::link::Link;

type GainQueue = VecDeque<GainPtr>;
type ModulationQueue = VecDeque<ModPtr>;

struct SendQueue {
    gain_q: GainQueue,
    modulation_q: ModulationQueue,
}
pub struct AsyncController<L: Link + 'static> {
    logic: Arc<Mutex<AUTDLogic<L>>>,
    is_running: Arc<RwLock<bool>>,
    build_gain_q: Arc<(Mutex<GainQueue>, Condvar)>,
    build_mod_q: Arc<(Mutex<ModulationQueue>, Condvar)>,
    send_q: Arc<(Mutex<SendQueue>, Condvar)>,
    build_gain_th_handle: Option<JoinHandle<()>>,
    build_mod_th_handle: Option<JoinHandle<()>>,
    send_th_handle: Option<JoinHandle<()>>,
}

impl<L: Link> AsyncController<L> {
    pub fn new(logic: Arc<Mutex<AUTDLogic<L>>>) -> Self {
        let send_q = Arc::new((
            Mutex::new(SendQueue {
                gain_q: GainQueue::new(),
                modulation_q: ModulationQueue::new(),
            }),
            Condvar::new(),
        ));
        let mut cnt = Self {
            logic,
            is_running: Arc::new(RwLock::new(true)),
            build_gain_q: Arc::new((Mutex::new(VecDeque::new()), Condvar::new())),
            build_mod_q: Arc::new((Mutex::new(VecDeque::new()), Condvar::new())),
            send_q,
            build_gain_th_handle: None,
            build_mod_th_handle: None,
            send_th_handle: None,
        };
        cnt.init_pipeline();
        cnt
    }

    pub fn append_gain(&mut self, gain: GainPtr) {
        let (build_lk, build_cvar) = &*self.build_gain_q;
        {
            let mut build_q = build_lk.lock().unwrap();
            build_q.push_back(gain);
        }
        build_cvar.notify_one();
    }

    pub fn append_modulation(&mut self, modulation: ModPtr) {
        let (build_lk, build_cvar) = &*self.build_mod_q;
        {
            let mut deq = build_lk.lock().unwrap();
            deq.push_back(modulation);
        }
        build_cvar.notify_one();
    }

    pub fn remaining_in_buffer(&self) -> usize {
        let (build_lk, _) = &*self.build_gain_q;
        let remain_build_gain = {
            let build_q = build_lk.lock().unwrap();
            build_q.len()
        };
        let (build_lk, _) = &*self.build_mod_q;
        let remain_build_mod = {
            let build_q = build_lk.lock().unwrap();
            build_q.len()
        };
        let (send_lk, _) = &*self.send_q;
        let remain_send = {
            let send_q = send_lk.lock().unwrap();
            send_q.gain_q.len() + send_q.modulation_q.len()
        };
        remain_build_gain + remain_build_mod + remain_send
    }

    pub fn flush(&mut self) {
        let (build_lk, _) = &*self.build_gain_q;
        {
            let mut build_q = build_lk.lock().unwrap();
            build_q.clear();
        }
        let (build_lk, _) = &*self.build_mod_q;
        {
            let mut build_q = build_lk.lock().unwrap();
            build_q.clear();
        }
        let (send_lk, _) = &*self.send_q;
        {
            let mut send_q = send_lk.lock().unwrap();
            send_q.gain_q.clear();
            send_q.modulation_q.clear();
        }
    }

    pub fn close(&mut self) {
        self.flush();
        if let Ok(mut run) = self.is_running.write() {
            *run = true;
        }
        if let Some(jh) = self.build_gain_th_handle.take() {
            let (_, build_cvar) = &*self.build_gain_q;
            build_cvar.notify_one();
            jh.join().unwrap();
        }

        if let Some(jh) = self.build_mod_th_handle.take() {
            let (_, build_cvar) = &*self.build_gain_q;
            build_cvar.notify_one();
            jh.join().unwrap();
        }

        if let Some(jh) = self.send_th_handle.take() {
            let (_, send_cvar) = &*self.send_q;
            send_cvar.notify_one();
            jh.join().unwrap();
        }
    }

    fn init_pipeline(&mut self) {
        // Build Gain thread
        let logic = self.logic.clone();
        let is_running = self.is_running.clone();
        let build_gain_q = self.build_gain_q.clone();
        let send_q = self.send_q.clone();
        self.build_gain_th_handle = Some(thread::spawn(move || {
            let (build_lk, build_cvar) = &*build_gain_q;
            loop {
                if let Ok(run) = is_running.read() {
                    if !*run {
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
                        let logic = logic.lock().unwrap();
                        logic.build_gain_ptr(&mut gain);
                        gain
                    }
                };

                let (send_lk, send_cvar) = &*send_q;
                {
                    let mut deq = send_lk.lock().unwrap();
                    deq.gain_q.push_back(gain);
                }
                send_cvar.notify_all();
            }
        }));

        // Build Modulation thread
        let logic = self.logic.clone();
        let is_running = self.is_running.clone();
        let build_mod_q = self.build_mod_q.clone();
        let send_q = self.send_q.clone();
        self.build_mod_th_handle = Some(thread::spawn(move || {
            let (build_lk, build_cvar) = &*build_mod_q;
            loop {
                if let Ok(run) = is_running.read() {
                    if !*run {
                        break;
                    }
                }
                let mut mod_q = build_lk.lock().unwrap();
                let modulation = match mod_q.pop_front() {
                    None => {
                        let _q = build_cvar.wait(mod_q).unwrap();
                        continue;
                    }
                    Some(mut modulation) => {
                        let logic = logic.lock().unwrap();
                        logic.build_modulation_ptr(&mut modulation);
                        modulation
                    }
                };

                let (send_lk, send_cvar) = &*send_q;
                {
                    let mut deq = send_lk.lock().unwrap();
                    deq.modulation_q.push_back(modulation);
                }
                send_cvar.notify_all();
            }
        }));

        // Send thread
        let logic = self.logic.clone();
        let is_running = self.is_running.clone();
        let send_q = self.send_q.clone();
        self.send_th_handle = Some(thread::spawn(move || {
            let (send_lk, send_cvar) = &*send_q;
            loop {
                if let Ok(open) = is_running.read() {
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
                        let mut logic = logic.lock().unwrap();
                        if logic.send_gain_mod_ptr(Some(g), None).is_err() {
                            if let Ok(mut run) = is_running.write() {
                                *run = false;
                            }
                            return;
                        }
                    }
                    (g, Some(m)) => {
                        let mut logic = logic.lock().unwrap();
                        if logic.send_gain_mod_ptr(g, Some(m)).is_err() {
                            if let Ok(mut run) = is_running.write() {
                                *run = false;
                            }
                            return;
                        }
                        if m.buffer().len() <= *m.sent() {
                            *m.sent() = 0;
                            send_buf.modulation_q.pop_front();
                        }
                    }
                }
            }
        }));
    }
}
