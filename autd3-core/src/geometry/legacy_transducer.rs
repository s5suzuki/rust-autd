/*
 * File: legacy_transducer.rs
 * Project: geometry
 * Created Date: 04/05/2022
 * Author: Shun Suzuki
 * -----
 * Last Modified: 31/05/2022
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2022 Shun Suzuki. All rights reserved.
 *
 */

use std::f64::consts::PI;

use autd3_driver::{LegacyDrive, NUM_TRANS_IN_UNIT};

use super::{DriveData, Transducer, Vector3};

pub struct LegacyDriveData {
    pub data: Vec<LegacyDrive>,
}

impl<T: Transducer> DriveData<T> for LegacyDriveData {
    fn new() -> Self {
        Self { data: vec![] }
    }

    fn init(&mut self, size: usize) {
        self.data.resize(
            size,
            LegacyDrive {
                phase: 0x00,
                duty: 0x00,
            },
        )
    }

    fn set_drive(&mut self, tr: &T, phase: f64, amp: f64) {
        self.data[tr.id()].duty = (510.0 * amp.asin() / PI) as u8;
        self.data[tr.id()].phase = (((phase * 256.0).round() as i32) & 0xFF) as u8;
    }

    fn copy_from(&mut self, dev_id: usize, src: &Self) {
        self.data[(dev_id * NUM_TRANS_IN_UNIT)..((dev_id + 1) * NUM_TRANS_IN_UNIT)]
            .copy_from_slice(
                &src.data[(dev_id * NUM_TRANS_IN_UNIT)..((dev_id + 1) * NUM_TRANS_IN_UNIT)],
            );
    }
}

pub struct LegacyTransducer {
    id: usize,
    pos: Vector3,
    x_direction: Vector3,
    y_direction: Vector3,
    z_direction: Vector3,
}

impl Transducer for LegacyTransducer {
    type D = LegacyDriveData;

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
        4096
    }

    fn frequency(&self) -> f64 {
        40e3
    }

    fn wavelength(&self, sound_speed: f64) -> f64 {
        sound_speed * 1e3 / 40e3
    }

    fn wavenumber(&self, sound_speed: f64) -> f64 {
        2.0 * PI * 40e3 / (sound_speed * 1e3)
    }

    fn pack_head(tx: &mut autd3_driver::TxDatagram) {
        autd3_driver::normal_legacy_head(tx);
    }

    fn pack_body(
        phase_sent: &mut bool,
        duty_sent: &mut bool,
        drives: &Self::D,
        tx: &mut autd3_driver::TxDatagram,
    ) -> anyhow::Result<()> {
        autd3_driver::normal_legacy_body(&drives.data, tx)?;
        *phase_sent = true;
        *duty_sent = true;
        Ok(())
    }
}
