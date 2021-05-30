/*
 * File: grouped.rs
 * Project: gain
 * Created Date: 30/05/2021
 * Author: Shun Suzuki
 * -----
 * Last Modified: 30/05/2021
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2021 Hapis Lab. All rights reserved.
 *
 */

use anyhow::Result;
use autd3_core::{gain::Gain, geometry::Geometry, hardware_defined::DataArray};
use autd3_traits::Gain;
use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
};

use crate::error::AutdError;

#[derive(Gain)]
pub struct Grouped<T: Hash + Eq> {
    data: Vec<DataArray>,
    built: bool,
    id_map: HashMap<T, Vec<usize>>,
    gain_map: HashMap<T, Box<dyn Gain>>,
}

impl<T: Hash + Eq> Grouped<T> {
    pub fn new(id_map: HashMap<T, Vec<usize>>, gain_map: HashMap<T, Box<dyn Gain>>) -> Self {
        let gids: HashSet<&T> = id_map.keys().collect();
        let gain_gids: HashSet<&T> = gain_map.keys().collect();

        assert!(gain_gids.is_subset(&gids));

        Self {
            id_map,
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

        for (group_id, device_ids) in &self.id_map {
            if let Some(gain) = &self.gain_map.get(group_id) {
                let d = gain.data();
                for device_id in device_ids {
                    if *device_id >= ndevice {
                        return Err(AutdError::GroupedOutOfRange(*device_id, ndevice).into());
                    }
                    self.data[*device_id] = d[*device_id];
                }
            }
        }

        self.built = true;
        Ok(())
    }
}
