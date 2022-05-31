/*
 * File: plane.rs
 * Project: gain
 * Created Date: 05/05/2022
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
pub struct Plane<T: Transducer> {
    props: GainProps<T>,
    power: f64,
    dir: Vector3,
}

impl<T: Transducer> Plane<T> {
    /// constructor
    ///
    /// # Arguments
    ///
    /// * `dir` - direction
    ///
    pub fn new(dir: Vector3) -> Self {
        Self::with_power(dir, 1.0)
    }

    /// constructor with power
    ///
    /// # Arguments
    ///
    /// * `dir` - direction
    /// * `power` - normalized power (from 0 to 1)
    ///
    pub fn with_power(dir: Vector3, power: f64) -> Self {
        Self {
            props: GainProps::new(),
            power,
            dir,
        }
    }
}

impl<T: Transducer> IGain<T> for Plane<T> {
    fn calc(&mut self, geometry: &Geometry<T>) -> anyhow::Result<()> {
        geometry.transducers().for_each(|tr| {
            let dist = self.dir.dot(tr.position());
            let phase = tr.align_phase_at(dist, geometry.sound_speed());
            self.props.drives.set_drive(tr, phase, self.power);
        });
        Ok(())
    }
}
