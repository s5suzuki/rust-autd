/*
 * File: null.rs
 * Project: gain
 * Created Date: 01/05/2022
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
    geometry::{DriveData, Geometry, Transducer},
};

use autd3_traits::Gain;

/// Gain to produce single focal point
#[derive(Gain)]
pub struct Null<T: Transducer> {
    props: GainProps<T>,
}

impl<T: Transducer> Null<T> {
    /// constructor
    pub fn new() -> Self {
        Self {
            props: GainProps::new(),
        }
    }
}

impl<T: Transducer> IGain<T> for Null<T> {
    fn calc(&mut self, geometry: &Geometry<T>) -> anyhow::Result<()> {
        geometry.transducers().for_each(|tr| {
            self.props.drives.set_drive(tr, 0.0, 0.0);
        });
        Ok(())
    }
}

impl<T: Transducer> Default for Null<T> {
    fn default() -> Self {
        Self::new()
    }
}
