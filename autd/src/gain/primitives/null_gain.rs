/*
 * File: null_gain.rs
 * Project: src
 * Created Date: 19/11/2019
 * Author: Shun Suzuki
 * -----
 * Last Modified: 31/12/2020
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2019 Hapis Lab. All rights reserved.
 *
 */

use crate::{consts::DataArray, geometry::Geometry};

use super::super::Gain;

/// Gain with no output
pub struct NullGain {
    data: Option<Vec<DataArray>>,
}

impl NullGain {
    pub fn create() -> Self {
        NullGain { data: None }
    }
}

impl Gain for NullGain {
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

        let num_devices = geometry.num_devices();
        let buf: DataArray = unsafe { std::mem::zeroed() };
        self.data = Some(vec![buf; num_devices]);
    }
}
