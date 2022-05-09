/*
 * File: grouped.rs
 * Project: gain
 * Created Date: 05/05/2022
 * Author: Shun Suzuki
 * -----
 * Last Modified: 05/05/2022
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2022 Hapis Lab. All rights reserved.
 *
 */

use std::collections::HashMap;

use autd3_core::{
    gain::{Gain, GainProps, IGain},
    geometry::{DriveData, Geometry, Transducer},
};

use autd3_traits::Gain;

use crate::error::AUTDError;

/// Gain to produce single focal point
#[derive(Gain)]
pub struct Grouped<T: Transducer> {
    props: GainProps<T>,
    gain_map: HashMap<usize, Box<dyn Gain<T>>>,
}

impl<T: Transducer> Grouped<T> {
    /// constructor
    pub fn new() -> Self {
        Self {
            props: GainProps::new(),
            gain_map: HashMap::new(),
        }
    }
}

impl<T: Transducer> IGain<T> for Grouped<T>
where
    Grouped<T>: Gain<T>,
{
    fn calc(&mut self, geometry: &Geometry<T>) -> anyhow::Result<()> {
        for gain in self.gain_map.values_mut() {
            gain.build(geometry)?;
        }

        self.gain_map.iter().try_for_each(|(dev_id, gain)| {
            if *dev_id >= geometry.num_devices() {
                return Err(AUTDError::GroupedOutOfRange(*dev_id, geometry.num_devices()).into());
            }

            self.props.drives.copy_from(*dev_id, gain.drives());

            Ok(())
        })
    }
}

impl<T: Transducer> Default for Grouped<T> {
    fn default() -> Self {
        Self::new()
    }
}
