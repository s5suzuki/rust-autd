/*
 * File: transducer.rs
 * Project: geometry
 * Created Date: 04/05/2022
 * Author: Shun Suzuki
 * -----
 * Last Modified: 01/06/2022
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2022 Shun Suzuki. All rights reserved.
 *
 */

use anyhow::Result;

use autd3_driver::TxDatagram;

use super::Vector3;

pub trait DriveData<T> {
    fn new() -> Self;
    fn init(&mut self, size: usize);
    fn set_drive(&mut self, tr: &T, phase: f64, amp: f64);
    fn copy_from(&mut self, dev_id: usize, src: &Self);
}

pub trait Transducer: Sized {
    type D: DriveData<Self>;

    fn new(
        id: usize,
        pos: Vector3,
        x_direction: Vector3,
        y_direction: Vector3,
        z_direction: Vector3,
    ) -> Self;
    fn align_phase_at(&self, dist: f64, sound_speed: f64) -> f64;
    fn position(&self) -> &Vector3;
    fn id(&self) -> usize;
    fn x_direction(&self) -> &Vector3;
    fn y_direction(&self) -> &Vector3;
    fn z_direction(&self) -> &Vector3;
    fn cycle(&self) -> u16;
    fn frequency(&self) -> f64;
    fn mod_delay(&self) -> u16;
    fn set_mod_delay(&mut self, delay: u16);
    fn wavelength(&self, sound_speed: f64) -> f64;
    fn wavenumber(&self, sound_speed: f64) -> f64;
    fn pack_head(tx: &mut TxDatagram);
    fn pack_body(
        phase_sent: &mut bool,
        duty_sent: &mut bool,
        drives: &Self::D,
        tx: &mut TxDatagram,
    ) -> Result<()>;
}
