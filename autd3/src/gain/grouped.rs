/*
 * File: grouped.rs
 * Project: gain
 * Created Date: 30/05/2021
 * Author: Shun Suzuki
 * -----
 * Last Modified: 16/12/2021
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2021 Hapis Lab. All rights reserved.
 *
 */

use anyhow::Result;
use autd3_core::{gain::Gain, geometry::Geometry, hardware_defined::Drive};
use autd3_traits::Gain;
use std::collections::HashMap;

use crate::error::AutdError;

#[derive(Gain)]
pub struct Grouped {
    data: Vec<Drive>,
    built: bool,
    gain_map: HashMap<usize, Box<dyn Gain>>,
}

impl Grouped {
    pub fn new(gain_map: HashMap<usize, Box<dyn Gain>>) -> Self {
        Self {
            gain_map,
            data: vec![],
            built: false,
        }
    }

    #[allow(clippy::unnecessary_wraps)]
    fn calc(&mut self, geometry: &Geometry) -> Result<()> {
        let ndevice = geometry.num_devices();

        for gain in self.gain_map.values_mut() {
            gain.build(geometry)?;
        }

        for (device_id, gain) in &self.gain_map {
            let d = gain.data();
            if *device_id >= ndevice {
                return Err(AutdError::GroupedOutOfRange(*device_id, ndevice).into());
            }
            self.data[*device_id] = d[*device_id];
        }
        Ok(())
    }
}
