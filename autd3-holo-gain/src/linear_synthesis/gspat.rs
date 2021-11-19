/*
 * File: gspat.rs
 * Project: linear_synthesis
 * Created Date: 29/05/2021
 * Author: Shun Suzuki
 * -----
 * Last Modified: 19/11/2021
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2021 Hapis Lab. All rights reserved.
 *
 */

use crate::{
    macros::{generate_propagation_matrix, set_from_complex_drive},
    Backend, Complex, MatrixXc, Transpose, VectorXc,
};
use anyhow::Result;
use autd3_core::{
    gain::Gain,
    geometry::{Geometry, Vector3},
    hardware_defined::{DataArray, NUM_TRANS_IN_UNIT},
};
use autd3_traits::Gain;
use nalgebra::ComplexField;
use std::marker::PhantomData;

/// Reference
/// * Diego Martinez Plasencia et al. "Gs-pat: high-speed multi-point sound-fields for phased arrays of transducers," ACMTrans-actions on Graphics (TOG), 39(4):138–1, 2020.
///
/// Not yet been implemented with GPU.
#[derive(Gain)]
pub struct GsPat<B: Backend> {
    data: Vec<DataArray>,
    built: bool,
    foci: Vec<Vector3>,
    amps: Vec<f64>,
    repeat: usize,
    backend: PhantomData<B>,
}

impl<B: Backend> GsPat<B> {
    pub fn new(foci: Vec<Vector3>, amps: Vec<f64>) -> Self {
        Self::with_param(foci, amps, 100)
    }

    pub fn with_param(foci: Vec<Vector3>, amps: Vec<f64>, repeat: usize) -> Self {
        assert!(foci.len() == amps.len());
        Self {
            data: vec![],
            built: false,
            foci,
            amps,
            repeat,
            backend: PhantomData,
        }
    }

    #[allow(clippy::many_single_char_names)]
    #[allow(clippy::unnecessary_wraps)]
    fn calc(&mut self, geometry: &Geometry) -> Result<()> {
        let m = self.foci.len();
        let n = geometry.num_devices() * NUM_TRANS_IN_UNIT;

        let g = generate_propagation_matrix(geometry, &self.foci);

        let denomi = g.column_sum();
        let b = g
            .map_with_location(|i, _, a| Complex::new(self.amps[i], 0.0) * a.conj() / denomi[i])
            .transpose();

        let mut r = MatrixXc::zeros(m, m);
        B::matrix_mul(
            Transpose::NoTrans,
            Transpose::NoTrans,
            Complex::new(1., 0.),
            &g,
            &b,
            Complex::new(0., 0.),
            &mut r,
        );

        let mut p = VectorXc::from_iterator(m, self.amps.iter().map(|&a| Complex::new(a, 0.)));

        let mut gamma = VectorXc::zeros(m);
        B::matrix_mul_vec(
            Transpose::NoTrans,
            Complex::new(1., 0.),
            &r,
            &p,
            Complex::new(0., 0.),
            &mut gamma,
        );
        for _ in 0..self.repeat {
            for i in 0..m {
                p[i] = gamma[i] / gamma[i].abs() * self.amps[i];
            }
            B::matrix_mul_vec(
                Transpose::NoTrans,
                Complex::new(1., 0.),
                &r,
                &p,
                Complex::new(0., 0.),
                &mut gamma,
            );
        }

        for i in 0..m {
            p[i] = gamma[i] / gamma[i].norm_sqr() * self.amps[i] * self.amps[i];
        }

        let mut q = VectorXc::zeros(n);
        B::matrix_mul_vec(
            Transpose::NoTrans,
            Complex::new(1., 0.),
            &b,
            &p,
            Complex::new(0., 0.),
            &mut q,
        );

        set_from_complex_drive(&mut self.data, &q, true, 1.0);
        Ok(())
    }
}
