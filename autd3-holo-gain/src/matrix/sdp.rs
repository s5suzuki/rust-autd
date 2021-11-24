/*
 * File: sdp.rs
 * Project: matrix
 * Created Date: 28/05/2021
 * Author: Shun Suzuki
 * -----
 * Last Modified: 24/11/2021
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
    gain::{Gain, GainData},
    geometry::{Geometry, Vector3},
    hardware_defined::NUM_TRANS_IN_UNIT,
};
use autd3_traits::Gain;
use nalgebra::ComplexField;
use rand::{thread_rng, Rng};
use std::{marker::PhantomData, ops::MulAssign};

/// Reference
/// * Inoue, Seki, Yasutoshi Makino, and Hiroyuki Shinoda. "Active touch perception produced by airborne ultrasonic haptic hologram." 2015 IEEE World Haptics Conference (WHC). IEEE, 2015.
#[derive(Gain)]
pub struct Sdp<B: Backend> {
    data: Vec<GainData>,
    built: bool,
    foci: Vec<Vector3>,
    amps: Vec<f64>,
    alpha: f64,
    lambda: f64,
    repeat: usize,
    normalize: bool,
    backend: PhantomData<B>,
}

impl<B: Backend> Sdp<B> {
    pub fn new(foci: Vec<Vector3>, amps: Vec<f64>) -> Self {
        Self::with_params(foci, amps, 1e-3, 0.9, 100, true)
    }

    pub fn with_params(
        foci: Vec<Vector3>,
        amps: Vec<f64>,
        alpha: f64,
        lambda: f64,
        repeat: usize,
        normalize: bool,
    ) -> Self {
        assert!(foci.len() == amps.len());
        Self {
            data: vec![],
            built: false,
            foci,
            amps,
            alpha,
            lambda,
            repeat,
            normalize,
            backend: PhantomData,
        }
    }

    #[allow(clippy::many_single_char_names)]
    #[allow(clippy::unnecessary_wraps)]
    fn calc(&mut self, geometry: &Geometry) -> Result<()> {
        let m = self.foci.len();
        let n = geometry.num_devices() * NUM_TRANS_IN_UNIT;

        let p = MatrixXc::from_diagonal(&VectorXc::from_iterator(
            m,
            self.amps.iter().map(|&a| Complex::new(a, 0.)),
        ));
        let b = generate_propagation_matrix(geometry, &self.foci);
        let mut pseudo_inv_b = MatrixXc::zeros(n, m);
        B::pseudo_inverse_svd(b.clone(), self.alpha, &mut pseudo_inv_b);

        let mut mm = MatrixXc::identity(m, m);
        B::matrix_mul(
            Transpose::NoTrans,
            Transpose::NoTrans,
            Complex::new(1., 0.),
            &b,
            &pseudo_inv_b,
            Complex::new(-1., 0.),
            &mut mm,
        );
        let mut tmp = MatrixXc::zeros(m, m);
        B::matrix_mul(
            Transpose::NoTrans,
            Transpose::NoTrans,
            Complex::new(1., 0.),
            &p,
            &mm,
            Complex::new(0., 0.),
            &mut tmp,
        );
        B::matrix_mul(
            Transpose::NoTrans,
            Transpose::NoTrans,
            Complex::new(1., 0.),
            &tmp,
            &p,
            Complex::new(0., 0.),
            &mut mm,
        );
        let mut x_mat = MatrixXc::identity(m, m);

        let mut rng = thread_rng();
        let zero = VectorXc::zeros(m);
        let mut x = VectorXc::zeros(m);

        fn set_bcd_result(mat: &mut MatrixXc, vec: &VectorXc, idx: usize) {
            let m = vec.len();
            mat.slice_mut((idx, 0), (1, idx))
                .copy_from(&vec.slice((0, 0), (idx, 1)).adjoint());
            mat.slice_mut((idx, idx + 1), (1, m - idx - 1))
                .copy_from(&vec.slice((0, 0), (m - idx - 1, 1)).adjoint());
            mat.slice_mut((0, idx), (idx, 1))
                .copy_from(&vec.slice((0, 0), (idx, 1)));
            mat.slice_mut((idx + 1, idx), (m - idx - 1, 1))
                .copy_from(&vec.slice((0, 0), (m - idx - 1, 1)));
        }

        for _ in 0..self.repeat {
            let ii = (m as f64 * rng.gen_range(0.0..1.0)) as usize;

            let mut mmc: VectorXc = mm.column(ii).into();
            mmc[ii] = Complex::new(0., 0.);

            B::matrix_mul_vec(
                Transpose::NoTrans,
                Complex::new(1., 0.),
                &x_mat,
                &mmc,
                Complex::new(0., 0.),
                &mut x,
            );
            let gamma = B::dot_c(&x, &mmc);
            if gamma.real() > 0.0 {
                x.mul_assign(Complex::new((self.lambda / gamma.real()).sqrt(), 0.));
                set_bcd_result(&mut x_mat, &x, ii);
            } else {
                set_bcd_result(&mut x_mat, &zero, ii);
            }
        }

        let u = B::max_eigen_vector(x_mat);

        let mut ut = VectorXc::zeros(m);
        B::matrix_mul_vec(
            Transpose::NoTrans,
            Complex::new(1., 0.),
            &p,
            &u,
            Complex::new(0., 0.),
            &mut ut,
        );

        let mut q = VectorXc::zeros(n);
        B::matrix_mul_vec(
            Transpose::NoTrans,
            Complex::new(1., 0.),
            &pseudo_inv_b,
            &ut,
            Complex::new(0., 0.),
            &mut q,
        );

        let max_coeff = B::max_coefficient_c(&q);
        set_from_complex_drive(&mut self.data, &q, self.normalize, max_coeff);
        Ok(())
    }
}
