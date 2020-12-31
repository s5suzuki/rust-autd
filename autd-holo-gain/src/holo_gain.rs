/*
 * File: holo_gain.rs
 * Project: gain
 * Created Date: 22/11/2019
 * Author: Shun Suzuki
 * -----
 * Last Modified: 31/12/2020
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2019 Hapis Lab. All rights reserved.
 * -----
 * The following algorithm is originally developed by Seki Inoue et al.
 * S. Inoue et al, "Active Touch Perception Produced by Airborne Ultrasonic Haptic Hologram," Proc. 2015 IEEE World Haptics Conference, pp.362-367, Northwestern University, Evanston, II, USA, June 22–26, 2015.
 *
 */

use autd::{
    consts::DataArray,
    gain::Gain,
    geometry::{Geometry, Vector3},
    utils::attenuation_coef,
    Float,
};

use crate::Optimizer;

pub struct HoloGain<O: Optimizer> {
    foci: Vec<Vector3>,
    amps: Vec<Float>,
    opt: O,
    data: Option<Vec<DataArray>>,
}

impl<O: Optimizer> HoloGain<O> {
    pub fn create(foci: Vec<Vector3>, amps: Vec<Float>, opt: O) -> Self {
        assert_eq!(foci.len(), amps.len());
        Self {
            foci,
            amps,
            opt,
            data: None,
        }
    }
}

impl<O: Optimizer> Gain for HoloGain<O> {
    fn get_data(&self) -> &[DataArray] {
        assert!(self.data.is_some());
        match &self.data {
            Some(data) => data,
            None => panic!(),
        }
    }

    #[allow(clippy::many_single_char_names)]
    fn build(&mut self, geometry: &Geometry) {
        if self.data.is_some() {
            return;
        }

        let num_devices = geometry.num_devices();
        let buf: DataArray = unsafe { std::mem::zeroed() };
        let mut data = vec![buf; num_devices];

        let temperature = 300.;
        let atten = attenuation_coef(40e3, 30., 1., 1., temperature);
        self.opt
            .optimize(geometry, &self.foci, &self.amps, atten, &mut data);

        self.data = Some(data);
    }
}
