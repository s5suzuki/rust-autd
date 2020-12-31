/*
 * File: stm_controller.rs
 * Project: controller
 * Created Date: 30/12/2020
 * Author: Shun Suzuki
 * -----
 * Last Modified: 31/12/2020
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2020 Hapis Lab. All rights reserved.
 *
 */

use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};

use autd_timer::Timer;

use super::{autd_logic::AUTDLogic, GainPtr};
use crate::{link::Link, Float};

pub struct STMController<L: Link> {
    logic: Arc<Mutex<AUTDLogic<L>>>,
    stm_gains: Arc<Mutex<VecDeque<GainPtr>>>,
    stm_timer: Timer,
}

impl<L: Link + 'static> STMController<L> {
    pub fn new(logic: Arc<Mutex<AUTDLogic<L>>>) -> Self {
        Self {
            logic,
            stm_gains: Arc::new(Mutex::new(VecDeque::new())),
            stm_timer: Timer::new(),
        }
    }

    pub fn append_stm_gain(&mut self, gains: GainPtr) {
        self.stop_stm();
        let mut stm_gains = self.stm_gains.lock().unwrap();
        stm_gains.push_back(gains);
    }

    pub fn append_stm_gains(&mut self, gains: Vec<GainPtr>) {
        self.stop_stm();
        let mut stm_gains = self.stm_gains.lock().unwrap();
        stm_gains.extend(gains);
    }

    pub fn start_stm(&mut self, freq: Float) {
        let len = { self.stm_gains.lock().unwrap().len() };
        assert!(len != 0);
        let itvl_ms = 1000. / freq / len as Float;

        let logic = self.logic.lock().unwrap();
        let dev_num = logic.geometry().num_devices();
        let is_silent = logic.silent_mode;
        let mut stm_gains = self.stm_gains.lock().unwrap();
        let mut body_q = Vec::<Vec<u8>>::new();
        for _ in 0..stm_gains.len() {
            if let Some(mut gain) = stm_gains.pop_front() {
                logic.build_gain_ptr(&mut gain);
                let (_, body) =
                    AUTDLogic::<L>::make_body_ptr(Some(gain), None, dev_num, is_silent, false);
                body_q.push(body);
            }
        }

        let mut idx = 0;
        let logic = self.logic.clone();
        self.stm_timer.start(
            move || {
                let body = &body_q[idx % len];
                if logic.lock().unwrap().send_data(&body).is_err() {
                    return;
                }
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

    pub fn close(&mut self) {
        self.finish_stm()
    }
}
