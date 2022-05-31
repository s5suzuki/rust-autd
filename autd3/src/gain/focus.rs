/*
 * File: focus.rs
 * Project: gain
 * Created Date: 28/04/2022
 * Author: Shun Suzuki
 * -----
 * Last Modified: 31/05/2022
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2022 Shun Suzuki. All rights reserved.
 *
 */

use autd3_core::{
    gain::{Gain, GainProps, IGain},
    geometry::{DriveData, Geometry, Transducer, Vector3},
};

use autd3_traits::Gain;

/// Gain to produce single focal point
#[derive(Gain)]
pub struct Focus<T: Transducer> {
    props: GainProps<T>,
    power: f64,
    pos: Vector3,
}

impl<T: Transducer> Focus<T> {
    /// constructor
    ///
    /// # Arguments
    ///
    /// * `pos` - position of focal point
    ///
    pub fn new(pos: Vector3) -> Self {
        Self::with_power(pos, 1.0)
    }

    /// constructor with duty
    ///
    /// # Arguments
    ///
    /// * `pos` - position of focal point
    /// * `power` - normalized power (from 0 to 1)
    ///
    pub fn with_power(pos: Vector3, power: f64) -> Self {
        Self {
            props: GainProps::new(),
            power,
            pos,
        }
    }
}

impl<T: Transducer> IGain<T> for Focus<T> {
    fn calc(&mut self, geometry: &Geometry<T>) -> anyhow::Result<()> {
        geometry.transducers().for_each(|tr| {
            let dist = (self.pos - tr.position()).norm();
            let phase = tr.align_phase_at(dist, geometry.sound_speed());
            self.props.drives.set_drive(tr, phase, self.power);
        });

        Ok(())
    }
}
