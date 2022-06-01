/*
 * File: normal_phase_transducer.rs
 * Project: geometry
 * Created Date: 31/05/2022
 * Author: Shun Suzuki
 * -----
 * Last Modified: 01/06/2022
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2022 Shun Suzuki. All rights reserved.
 *
 */

use std::f64::consts::PI;

use anyhow::{Ok, Result};

use autd3_driver::{Duty, Phase, FPGA_CLK_FREQ, MAX_CYCLE, NUM_TRANS_IN_UNIT};

use crate::{
    error::AUTDInternalError,
    interface::{DatagramBody, Empty, Filled, Sendable},
};

use super::{DriveData, Geometry, Transducer, Vector3};

pub struct NormalPhaseDriveData {
    pub phases: Vec<Phase>,
}

impl<T: Transducer> DriveData<T> for NormalPhaseDriveData {
    fn new() -> Self {
        Self { phases: vec![] }
    }

    fn init(&mut self, size: usize) {
        self.phases.resize(size, Phase { phase: 0x0000 });
    }

    fn set_drive(&mut self, tr: &T, phase: f64, _amp: f64) {
        self.phases[tr.id()].set(phase, tr.cycle());
    }

    fn copy_from(&mut self, dev_id: usize, src: &Self) {
        self.phases[(dev_id * NUM_TRANS_IN_UNIT)..((dev_id + 1) * NUM_TRANS_IN_UNIT)]
            .copy_from_slice(
                &src.phases[(dev_id * NUM_TRANS_IN_UNIT)..((dev_id + 1) * NUM_TRANS_IN_UNIT)],
            );
    }
}

pub struct NormalPhaseTransducer {
    id: usize,
    pos: Vector3,
    x_direction: Vector3,
    y_direction: Vector3,
    z_direction: Vector3,
    cycle: u16,
    mod_delay: u16,
}

impl Transducer for NormalPhaseTransducer {
    type D = NormalPhaseDriveData;

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
            mod_delay: 0,
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

    fn mod_delay(&self) -> u16 {
        self.mod_delay
    }

    fn set_mod_delay(&mut self, delay: u16) {
        self.mod_delay = delay;
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
        autd3_driver::normal_phase_body(&drives.phases, tx)?;
        *phase_sent = true;
        *duty_sent = true;
        Ok(())
    }

    fn wavelength(&self, sound_speed: f64) -> f64 {
        sound_speed * 1e3 / self.frequency()
    }

    fn wavenumber(&self, sound_speed: f64) -> f64 {
        2.0 * PI * self.frequency() / (sound_speed * 1e3)
    }
}

impl NormalPhaseTransducer {
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

pub struct Amplitudes {
    pub duties: Vec<Duty>,
    sent: bool,
}

impl Amplitudes {
    pub fn uniform(geometry: &Geometry<NormalPhaseTransducer>, amp: f64) -> Self {
        let mut duties = vec![];
        duties.resize(geometry.num_transducers(), Duty { duty: 0x0000 });
        for (d, tr) in duties.iter_mut().zip(geometry.transducers()) {
            d.set(amp, tr.cycle());
        }
        Self {
            duties,
            sent: false,
        }
    }

    pub fn none(geometry: &Geometry<NormalPhaseTransducer>) -> Self {
        Self::uniform(geometry, 0.0)
    }
}

impl DatagramBody<NormalPhaseTransducer> for Amplitudes {
    fn init(&mut self) -> Result<()> {
        self.sent = false;
        Ok(())
    }

    fn pack(
        &mut self,
        _geometry: &Geometry<NormalPhaseTransducer>,
        tx: &mut autd3_driver::TxDatagram,
    ) -> Result<()> {
        autd3_driver::normal_head(tx);
        if DatagramBody::<NormalPhaseTransducer>::is_finished(self) {
            return Ok(());
        }
        self.sent = true;
        autd3_driver::normal_duty_body(&self.duties, tx)?;
        Ok(())
    }

    fn is_finished(&self) -> bool {
        self.sent
    }
}

impl Sendable<NormalPhaseTransducer> for Amplitudes {
    type H = Empty;
    type B = Filled;

    fn init(&mut self) -> Result<()> {
        DatagramBody::<NormalPhaseTransducer>::init(self)
    }

    fn pack(
        &mut self,
        _msg_id: u8,
        geometry: &Geometry<NormalPhaseTransducer>,
        tx: &mut autd3_driver::TxDatagram,
    ) -> Result<()> {
        DatagramBody::<NormalPhaseTransducer>::pack(self, geometry, tx)
    }

    fn is_finished(&self) -> bool {
        DatagramBody::<NormalPhaseTransducer>::is_finished(self)
    }
}
