/*
 * File: trans_test.rs
 * Project: gain
 * Created Date: 09/05/2022
 * Author: Shun Suzuki
 * -----
 * Last Modified: 31/05/2022
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2022 Shun Suzuki. All rights reserved.
 *
 */

use std::collections::HashMap;

use autd3_core::{
    gain::{Gain, GainProps, IGain},
    geometry::{DriveData, Geometry, Transducer},
};

use autd3_traits::Gain;

/// Gain to produce single focal point
#[derive(Gain, Default)]
pub struct TransducerTest<T: Transducer> {
    props: GainProps<T>,
    test_drive: HashMap<usize, (f64, f64)>,
}

impl<T: Transducer> TransducerTest<T> {
    /// constructor
    pub fn new() -> Self {
        Self {
            props: GainProps::default(),
            test_drive: HashMap::new(),
        }
    }

    pub fn set(&mut self, id: usize, phase: f64, power: f64) {
        self.test_drive.insert(id, (phase, power));
    }
}

impl<T: Transducer> IGain<T> for TransducerTest<T> {
    fn calc(&mut self, geometry: &Geometry<T>) -> anyhow::Result<()> {
        geometry.transducers().for_each(|tr| {
            if let Some((phase, power)) = self.test_drive.get(&tr.id()) {
                self.props.drives.set_drive(tr, *phase, *power);
            } else {
                self.props.drives.set_drive(tr, 0., 0.);
            }
        });

        Ok(())
    }
}
