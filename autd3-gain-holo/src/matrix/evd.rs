/*
 * File: evd.rs
 * Project: matrix
 * Created Date: 29/05/2021
 * Author: Shun Suzuki
 * -----
 * Last Modified: 31/05/2022
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2021 Shun Suzuki. All rights reserved.
 *
 */

use crate::{
    error::HoloError, macros::generate_propagation_matrix, Backend, Complex, MatrixXc, Transpose,
    VectorXc,
};
use anyhow::Result;
use autd3_core::{
    gain::{Gain, GainProps, IGain},
    geometry::{DriveData, Geometry, Transducer, Vector3},
    NUM_TRANS_IN_UNIT,
};
use autd3_traits::Gain;
use nalgebra::ComplexField;
use std::{f64::consts::PI, marker::PhantomData};

/// Reference
/// * Long, Benjamin, et al. "Rendering volumetric haptic shapes in mid-air using ultrasound." ACM Transactions on Graphics (TOG) 33.6 (2014): 1-10.
#[derive(Gain)]
pub struct EVD<B: Backend, T: Transducer> {
    props: GainProps<T>,
    foci: Vec<Vector3>,
    amps: Vec<f64>,
    gamma: f64,
    normalize: bool,
    backend: PhantomData<B>,
}

impl<B: Backend, T: Transducer> EVD<B, T> {
    pub fn new(foci: Vec<Vector3>, amps: Vec<f64>) -> Self {
        Self::with_params(foci, amps, 1.0, true)
    }

    pub fn with_params(foci: Vec<Vector3>, amps: Vec<f64>, gamma: f64, normalize: bool) -> Self {
        assert!(foci.len() == amps.len());
        Self {
            props: GainProps::default(),
            foci,
            amps,
            gamma,
            normalize,
            backend: PhantomData,
        }
    }
}

impl<B: Backend, T: Transducer> IGain<T> for EVD<B, T> {
    fn calc(&mut self, geometry: &Geometry<T>) -> Result<()> {
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
        geometry.transducers().for_each(move |tr| {
            let phase = gtf[tr.id()].argument() / (2.0 * PI) + 0.5;
            let power = if self.normalize {
                1.0
            } else {
                gtf[tr.id()].abs() / max_coeff
            };
            self.props.drives.set_drive(tr, phase, power);
        });

        Ok(())
    }
}
