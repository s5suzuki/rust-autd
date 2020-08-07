/*
 * File: null_gain.rs
 * Project: src
 * Created Date: 19/11/2019
 * Author: Shun Suzuki
 * -----
 * Last Modified: 07/08/2020
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2019 Hapis Lab. All rights reserved.
 *
 */

use crate::consts::NUM_TRANS_IN_UNIT;
use crate::geometry::Geometry;

use super::super::Gain;

/// Gain with no output
pub struct NullGain {
    data: Option<Vec<u8>>,
}

impl NullGain {
    pub fn create() -> Box<NullGain> {
        Box::new(NullGain { data: None })
    }
}

impl Gain for NullGain {
    fn get_data(&self) -> &[u8] {
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
        let num_trans = NUM_TRANS_IN_UNIT * num_devices;
        self.data = Some(vec![0x00; num_trans * 2]);
    }
}
