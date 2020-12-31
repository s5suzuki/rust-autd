/*
 * File: grouped_gain.rs
 * Project: gain
 * Created Date: 02/12/2019
 * Author: Shun Suzuki
 * -----
 * Last Modified: 31/12/2020
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2019 Hapis Lab. All rights reserved.
 *
 */

use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
};

use crate::{consts::DataArray, gain::Gain, geometry::Geometry};

pub struct GroupedGain<T: Sized + Send + Hash + Eq> {
    id_map: HashMap<T, Vec<usize>>,
    gain_map: HashMap<T, Box<dyn Gain>>,
    data: Option<Vec<DataArray>>,
}

impl<T: Sized + Send + Hash + Eq> GroupedGain<T> {
    pub fn create(
        id_map: HashMap<T, Vec<usize>>,
        gain_map: HashMap<T, Box<dyn Gain>>,
    ) -> GroupedGain<T> {
        let gids: HashSet<&T> = id_map.keys().collect();
        let gain_gids: HashSet<&T> = gain_map.keys().collect();

        assert!(gain_gids.is_subset(&gids));

        GroupedGain {
            id_map,
            gain_map,
            data: None,
        }
    }
}

impl<T: Sized + Send + Hash + Eq> Gain for GroupedGain<T> {
    fn get_data(&self) -> &[DataArray] {
        assert!(self.data.is_some());
        match &self.data {
            Some(data) => data,
            None => panic!(),
        }
    }

    fn build(&mut self, geometry: &Geometry) {
        if self.data.is_some() {
            return;
        }

        let ndevice = geometry.num_devices();
        let mut data = Vec::with_capacity(ndevice);
        unsafe {
            data.set_len(ndevice);
        }

        for gain in self.gain_map.values_mut() {
            gain.build(geometry);
        }

        for (group_id, device_ids) in &self.id_map {
            if let Some(gain) = &self.gain_map.get(group_id) {
                let d = gain.get_data();
                for device_id in device_ids {
                    if *device_id >= ndevice {
                        panic!(
                        "You specified device id ({}) in GroupedGain, but only {} AUTDs are connected.",
                        *device_id, ndevice
                    );
                    }
                    data[*device_id] = d[*device_id];
                }
            }
        }

        self.data = Some(data);
    }
}
