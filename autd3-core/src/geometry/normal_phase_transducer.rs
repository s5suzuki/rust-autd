/*
 * File: normal_phase_transducer.rs
 * Project: geometry
 * Created Date: 31/05/2022
 * Author: Shun Suzuki
 * -----
 * Last Modified: 28/07/2022
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2022 Shun Suzuki. All rights reserved.
 *
 */

use std::f64::consts::PI;

use anyhow::Result;

use autd3_driver::{Drive, FPGA_CLK_FREQ, MAX_CYCLE};

use crate::{
    error::AUTDInternalError,
    interface::{DatagramBody, Empty, Filled, Sendable},
};

use super::{Geometry, Transducer, Vector3};

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
        drives: &[Drive],
        tx: &mut autd3_driver::TxDatagram,
    ) -> anyhow::Result<()> {
        autd3_driver::normal_phase_body(drives, tx)?;
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
    pub drives: Vec<Drive>,
    sent: bool,
}

impl Amplitudes {
    pub fn uniform(geometry: &Geometry<NormalPhaseTransducer>, amp: f64) -> Self {
        Self {
            drives: geometry
                .transducers()
                .map(|tr| Drive {
                    phase: 0.0,
                    amp,
                    cycle: tr.cycle,
                })
                .collect(),
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
        autd3_driver::normal_duty_body(&self.drives, tx)?;
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
