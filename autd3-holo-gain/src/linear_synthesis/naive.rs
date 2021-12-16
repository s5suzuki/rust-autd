/*
 * File: naive.rs
 * Project: linear_synthesis
 * Created Date: 28/05/2021
 * Author: Shun Suzuki
 * -----
 * Last Modified: 16/12/2021
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2021 Hapis Lab. All rights reserved.
 *
 */

use crate::{
    macros::{generate_propagation_matrix, set_from_complex_drive},
    Backend, Complex, Transpose, VectorXc,
};
use anyhow::Result;
use autd3_core::{
    gain::Gain,
    geometry::{Geometry, Vector3},
    hardware_defined::{Drive, NUM_TRANS_IN_UNIT},
};
use autd3_traits::Gain;
use std::marker::PhantomData;

/// Naive linear synthesis
#[derive(Gain)]
pub struct Naive<B: Backend> {
    data: Vec<Drive>,
    built: bool,
    foci: Vec<Vector3>,
    amps: Vec<f64>,
    backend: PhantomData<B>,
}

impl<B: Backend> Naive<B> {
    pub fn new(foci: Vec<Vector3>, amps: Vec<f64>) -> Self {
        assert!(foci.len() == amps.len());
        Self {
            data: vec![],
            built: false,
            foci,
            amps,
            backend: PhantomData,
        }
    }

    #[allow(clippy::many_single_char_names)]
    #[allow(clippy::unnecessary_wraps)]
    fn calc(&mut self, geometry: &Geometry) -> Result<()> {
        let m = self.foci.len();
        let n = geometry.num_devices() * NUM_TRANS_IN_UNIT;

        let g = generate_propagation_matrix(geometry, &self.foci);
        let p = VectorXc::from_iterator(m, self.amps.iter().map(|&a| Complex::new(a, 0.0)));
        let mut q = VectorXc::zeros(n);
        B::matrix_mul_vec(
            Transpose::ConjTrans,
            Complex::new(1.0, 0.0),
            &g,
            &p,
            Complex::new(0.0, 0.0),
            &mut q,
        );

        set_from_complex_drive(&mut self.data, &q, true, 1.0);
        Ok(())
    }
}
