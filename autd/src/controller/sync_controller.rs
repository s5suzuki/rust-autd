/*
 * File: sync_controller.rs
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
    error::Error,
    sync::{Arc, Mutex},
};

use sequence::PointSequence;

use super::autd_logic::AUTDLogic;
use crate::{
    gain::Gain,
    link::Link,
    modulation::Modulation,
    prelude::{NoModulation, NullGain},
    sequence,
};

pub struct SyncController<L: Link> {
    logic: Arc<Mutex<AUTDLogic<L>>>,
}

impl<L: Link> SyncController<L> {
    pub fn new(logic: Arc<Mutex<AUTDLogic<L>>>) -> Self {
        Self { logic }
    }

    pub fn append_gain<G: Gain>(&mut self, g: &mut G) -> Result<(), Box<dyn Error>> {
        let mut logic = self.logic.lock().unwrap();
        logic.build_gain(g);
        logic.send_gain_mod::<G, NoModulation>(Some(g), None)?;
        Ok(())
    }

    pub fn append_gain_blocking<G: Gain>(&mut self, g: &mut G) -> Result<(), Box<dyn Error>> {
        let mut logic = self.logic.lock().unwrap();
        logic.build_gain(g);
        logic.send_gain_mod_blocking::<G, NoModulation>(Some(g), None)?;
        Ok(())
    }

    pub fn append_modulation<M: Modulation>(&mut self, m: &mut M) -> Result<(), Box<dyn Error>> {
        let mut logic = self.logic.lock().unwrap();
        logic.build_modulation(m);
        while *m.sent() < m.buffer().len() {
            logic.send_gain_mod_blocking::<NullGain, M>(None, Some(m))?;
        }
        Ok(())
    }

    pub fn append_seq(&mut self, seq: &mut PointSequence) -> Result<(), Box<dyn Error>> {
        let mut logic = self.logic.lock().unwrap();
        while *seq.sent() < seq.control_points().len() {
            logic.send_seq_blocking(seq)?;
        }
        logic.calibrate_seq()?;
        Ok(())
    }
}
