/*
 * File: evd.rs
 * Project: matrix
 * Created Date: 29/05/2021
 * Author: Shun Suzuki
 * -----
 * Last Modified: 02/10/2021
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2021 Hapis Lab. All rights reserved.
 *
 */

use crate::{
    error::HoloError,
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
/// * Long, Benjamin, et al. "Rendering volumetric haptic shapes in mid-air using ultrasound." ACM Transactions on Graphics (TOG) 33.6 (2014): 1-10.
#[derive(Gain)]
pub struct Evd<B: Backend> {
    data: Vec<DataArray>,
    built: bool,
    foci: Vec<Vector3>,
    amps: Vec<f64>,
    gamma: f64,
    normalize: bool,
    backend: PhantomData<B>,
}

impl<B: Backend> Evd<B> {
    pub fn new(foci: Vec<Vector3>, amps: Vec<f64>) -> Self {
        Self::with_params(foci, amps, 1.0, true)
    }

    pub fn with_params(foci: Vec<Vector3>, amps: Vec<f64>, gamma: f64, normalize: bool) -> Self {
        assert!(foci.len() == amps.len());
        Self {
            data: vec![],
            built: false,
            foci,
            amps,
            gamma,
            normalize,
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
        let x = g
            .map_with_location(|i, _, a| Complex::new(self.amps[i], 0.0) * a.conj() / denomi[i])
            .transpose();

        let mut r = MatrixXc::zeros(m, m);
        B::matrix_mul(
            Transpose::NoTrans,
            Transpose::NoTrans,
            Complex::new(1., 0.),
            &g,
            &x,
            Complex::new(0., 0.),
            &mut r,
        );
        let max_ev = B::max_eigen_vector(r);

        let sigma = MatrixXc::from_diagonal(&VectorXc::from_iterator(
            n,
            g.column_iter()
                .map(|col| {
                    col.iter()
                        .zip(self.amps.iter())
                        .map(|(a, &amp)| a.abs() * amp)
                        .sum()
                })
                .map(|s: f64| Complex::new((s / m as f64).sqrt().powf(self.gamma), 0.0)),
        ));

        let gr = B::concat_row(g, &sigma);
        let f = VectorXc::from_iterator(
            m + n,
            self.amps
                .iter()
                .zip(max_ev.iter())
                .map(|(amp, &e)| amp * e / e.abs())
                .chain((0..n).map(|_| Complex::new(0., 0.))),
        );

        let mut gtg = MatrixXc::zeros(n, n);
        B::matrix_mul(
            Transpose::ConjTrans,
            Transpose::NoTrans,
            Complex::new(1., 0.),
            &gr,
            &gr,
            Complex::new(0., 0.),
            &mut gtg,
        );

        let mut gtf = VectorXc::zeros(n);
        B::matrix_mul_vec(
            Transpose::ConjTrans,
            Complex::new(1., 0.),
            &gr,
            &f,
            Complex::new(0., 0.),
            &mut gtf,
        );

        if !B::solve_ch(gtg, &mut gtf) {
            return Err(HoloError::SolveFailed.into());
        }

        let max_coeff = B::max_coefficient_c(&gtf);
        set_from_complex_drive(&mut self.data, &gtf, self.normalize, max_coeff);

        self.built = true;
        Ok(())
    }
}
