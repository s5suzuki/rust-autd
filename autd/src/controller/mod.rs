/*
 * File: mod.rs
 * Project: controller
 * Created Date: 07/08/2020
 * Author: Shun Suzuki
 * -----
 * Last Modified: 31/12/2020
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2020 Hapis Lab. All rights reserved.
 *
 */

mod async_controller;
mod autd_logic;
mod stm_controller;
mod sync_controller;

use std::{
    error::Error,
    sync::{Arc, Mutex, MutexGuard},
};

use async_controller::AsyncController;
use autd_logic::AUTDLogic;
use sync_controller::SyncController;

use crate::{
    consts::DataArray,
    core::{configuration::Configuration, FirmwareInfo},
    gain::Gain,
    geometry::Geometry,
    link::Link,
    modulation::Modulation,
    prelude::NullGain,
    sequence::PointSequence,
    Float,
};

use self::stm_controller::STMController;

type GainPtr = Box<dyn Gain>;
type ModPtr = Box<dyn Modulation>;

pub struct AUTD<L: Link + 'static> {
    logic: Arc<Mutex<AUTDLogic<L>>>,
    sync_cnt: SyncController<L>,
    async_cnt: AsyncController<L>,
    stm_cnt: STMController<L>,
}

impl<L: Link + 'static> AUTD<L> {
    pub fn open(geometry: Geometry, mut link: L) -> Result<Self, Box<dyn Error>> {
        link.open()?;
        let logic = AUTDLogic::new(geometry, link);
        let logic = Arc::new(Mutex::new(logic));
        Ok(Self {
            logic: logic.clone(),
            sync_cnt: SyncController::new(logic.clone()),
            async_cnt: AsyncController::new(logic.clone()),
            stm_cnt: STMController::new(logic),
        })
    }

    pub fn is_open(&self) -> bool {
        self.logic().is_open()
    }

    pub fn silent_mode(&self) -> bool {
        self.logic().silent_mode
    }

    pub fn remaining_in_buffer(&self) -> usize {
        self.async_cnt.remaining_in_buffer()
    }

    pub fn logic(&self) -> MutexGuard<'_, AUTDLogic<L>> {
        self.logic.lock().unwrap()
    }

    pub fn set_silent_mode(&mut self, silent: bool) {
        self.logic().silent_mode = silent;
    }

    pub fn set_delay(&mut self, delays: &[DataArray]) -> Result<(), Box<dyn Error>> {
        self.logic().set_delay(delays)
    }

    pub fn clear(&mut self) -> Result<bool, Box<dyn Error>> {
        self.logic().clear()
    }

    pub fn calibrate(&mut self, config: Configuration) -> Result<bool, Box<dyn Error>> {
        self.logic().calibrate(config)
    }

    pub fn close(mut self) -> Result<bool, Box<dyn Error>> {
        self.close_impl()
    }

    pub fn close_impl(&mut self) -> Result<bool, Box<dyn Error>> {
        if !self.is_open() {
            return Ok(true);
        }
        self.stm_cnt.close();
        self.async_cnt.close();
        self.stop()?;
        if !self.clear()? {
            return Ok(false);
        }
        self.logic().close()
    }

    pub fn flush(&mut self) {
        self.async_cnt.flush()
    }

    pub fn stop(&mut self) -> Result<(), Box<dyn Error>> {
        let mut g = NullGain::create();
        self.append_gain_sync(&mut g)
    }

    pub fn append_gain<G: Gain + 'static>(&mut self, gain: G) {
        self.async_cnt.append_gain(Box::new(gain));
    }

    pub fn append_gain_sync<G: Gain>(&mut self, gain: &mut G) -> Result<(), Box<dyn Error>> {
        self.append_gain_sync_with_wait(gain, false)
    }

    pub fn append_gain_sync_with_wait<G: Gain>(
        &mut self,
        gain: &mut G,
        wait: bool,
    ) -> Result<(), Box<dyn Error>> {
        self.stm_cnt.stop_stm();
        if wait {
            self.sync_cnt.append_gain_blocking(gain)
        } else {
            self.sync_cnt.append_gain(gain)
        }
    }

    pub fn append_modulation<M: Modulation + 'static>(&mut self, m: M) {
        self.async_cnt.append_modulation(Box::new(m));
    }

    pub fn append_modulation_sync<M: Modulation>(
        &mut self,
        m: &mut M,
    ) -> Result<(), Box<dyn Error>> {
        self.sync_cnt.append_modulation(m)
    }

    pub fn append_sequence(&mut self, seq: &mut PointSequence) -> Result<(), Box<dyn Error>> {
        self.sync_cnt.append_seq(seq)
    }

    pub fn firmware_info_list(&mut self) -> Result<Vec<FirmwareInfo>, Box<dyn Error>> {
        self.logic().firmware_info_list()
    }

    pub fn append_stm_gain<G: Gain + 'static>(&mut self, gain: G) {
        self.stm_cnt.append_stm_gain(Box::new(gain))
    }

    pub fn append_stm_gains(&mut self, gains: Vec<GainPtr>) {
        self.stm_cnt.append_stm_gains(gains)
    }

    pub fn start_stm(&mut self, freq: Float) {
        self.stm_cnt.start_stm(freq)
    }

    pub fn stop_stm(&mut self) {
        self.stm_cnt.stop_stm()
    }

    pub fn finish_stm(&mut self) {
        self.stm_cnt.finish_stm()
    }
}

impl<L: Link + 'static> Drop for AUTD<L> {
    fn drop(&mut self) {
        let _ = self.close_impl();
    }
}
