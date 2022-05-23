/*
 * File: normal_transducer.rs
 * Project: geometry
 * Created Date: 04/05/2022
 * Author: Shun Suzuki
 * -----
 * Last Modified: 23/05/2022
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2022 Hapis Lab. All rights reserved.
 *
 */

use std::f64::consts::PI;

use anyhow::{Ok, Result};

use autd3_driver::{Duty, Phase, FPGA_CLK_FREQ, MAX_CYCLE, NUM_TRANS_IN_UNIT};

use crate::error::AUTDInternalError;

use super::{DriveData, Transducer, Vector3};

pub struct NormalDriveData {
    pub phases: Vec<Phase>,
    pub duties: Vec<Duty>,
}

impl<T: Transducer> DriveData<T> for NormalDriveData {
    fn new() -> Self {
        Self {
            phases: vec![],
            duties: vec![],
        }
    }

    fn init(&mut self, size: usize) {
        self.phases.resize(size, Phase { phase: 0x0000 });
        self.duties.resize(size, Duty { duty: 0x0000 });
    }

    fn set_drive(&mut self, tr: &T, phase: f64, amp: f64) {
        self.duties[tr.id()].duty = (tr.cycle() as f64 * amp.asin() / PI) as _;
        self.phases[tr.id()].phase =
            ((phase * tr.cycle() as f64).round() as i32).rem_euclid(tr.cycle() as i32) as _;
    }

    fn copy_from(&mut self, dev_id: usize, src: &Self) {
        self.duties[(dev_id * NUM_TRANS_IN_UNIT)..((dev_id + 1) * NUM_TRANS_IN_UNIT)]
            .copy_from_slice(
                &src.duties[(dev_id * NUM_TRANS_IN_UNIT)..((dev_id + 1) * NUM_TRANS_IN_UNIT)],
            );
        self.phases[(dev_id * NUM_TRANS_IN_UNIT)..((dev_id + 1) * NUM_TRANS_IN_UNIT)]
            .copy_from_slice(
                &src.phases[(dev_id * NUM_TRANS_IN_UNIT)..((dev_id + 1) * NUM_TRANS_IN_UNIT)],
            );
    }
}

pub struct NormalTransducer {
    id: usize,
    pos: Vector3,
    x_direction: Vector3,
    y_direction: Vector3,
    z_direction: Vector3,
    cycle: u16,
}

impl Transducer for NormalTransducer {
    type D = NormalDriveData;

    fn new(
        id: usize,
        pos: Vector3,
        x_direction: Vector3,
        y_direction: Vector3,
        z_direction: Vector3,
    ) -> Self {
        Self {
            id,
            pos,
            x_direction,
            y_direction,
            z_direction,
            cycle: 4096,
        }
    }
    fn align_phase_at(&self, dist: f64, sound_speed: f64) -> f64 {
        let wavelength = sound_speed * 1e3 / self.frequency();
        dist / wavelength
    }

    fn position(&self) -> &Vector3 {
        &self.pos
    }

    fn id(&self) -> usize {
        self.id
    }

    fn x_direction(&self) -> &Vector3 {
        &self.x_direction
    }

    fn y_direction(&self) -> &Vector3 {
        &self.y_direction
    }

    fn z_direction(&self) -> &Vector3 {
        &self.z_direction
    }

    fn cycle(&self) -> u16 {
        self.cycle
    }

    fn frequency(&self) -> f64 {
        FPGA_CLK_FREQ as f64 / self.cycle as f64
    }

    fn pack_head(tx: &mut autd3_driver::TxDatagram) {
        autd3_driver::normal_head(tx);
    }

    fn pack_body(
        phase_sent: &mut bool,
        duty_sent: &mut bool,
        drives: &Self::D,
        tx: &mut autd3_driver::TxDatagram,
    ) -> anyhow::Result<()> {
        if !*phase_sent {
            autd3_driver::normal_phase_body(&drives.phases, tx)?;
            *phase_sent = true;
        } else {
            autd3_driver::normal_duty_body(&drives.duties, tx)?;
            *duty_sent = true;
        }
        Ok(())
    }

    fn wavelength(&self, sound_speed: f64) -> f64 {
        sound_speed * 1e3 / self.frequency()
    }

    fn wavenumber(&self, sound_speed: f64) -> f64 {
        2.0 * PI * self.frequency() / (sound_speed * 1e3)
    }
}

impl NormalTransducer {
    pub fn set_cycle(&mut self, cycle: u16) -> Result<()> {
        if cycle > MAX_CYCLE {
            return Err(AUTDInternalError::CycleOutOfRange(cycle).into());
        }
        self.cycle = cycle;
        Ok(())
    }

    pub fn set_frequency(&mut self, freq: f64) -> Result<()> {
        let cycle = (FPGA_CLK_FREQ as f64 / freq).round() as u16;
        self.set_cycle(cycle)
    }
}
