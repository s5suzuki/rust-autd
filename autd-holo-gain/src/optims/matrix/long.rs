/*
 * File: long.rs
 * Project: multiple_foci
 * Created Date: 22/09/2020
 * Author: Shun Suzuki
 * -----
 * Last Modified: 31/12/2020
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2020 Hapis Lab. All rights reserved.
 *
 */

use crate::optims::{macros::*, Optimizer};
use autd::{
    consts::{DataArray, NUM_TRANS_IN_UNIT},
    geometry::Geometry,
    prelude::Vector3,
    Float, PI,
};
use na::ComplexField;
use num_traits::pow::Pow;

/// Reference
/// * Long, Benjamin, et al. "Rendering volumetric haptic shapes in mid-air using ultrasound." ACM Transactions on Graphics (TOG) 33.6 (2014): 1-10.
pub struct Long {
    pub gamma: Float,
    pub normalize: bool,
}

impl Long {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for Long {
    fn default() -> Self {
        Self {
            gamma: 1.0,
            normalize: true,
        }
    }
}

impl Optimizer for Long {
    #[allow(non_snake_case)]
    #[allow(clippy::many_single_char_names)]
    fn optimize(
        &self,
        geometry: &Geometry,
        foci: &[Vector3],
        amps: &[Float],
        atten: Float,
        data: &mut [DataArray],
    ) {
        let m = foci.len();
        let n = geometry.num_devices() * NUM_TRANS_IN_UNIT;

        let G = generate_propagation_matrix(geometry, atten, foci);

        let denomi = G.column_sum();
        let X = G
            .map_with_location(|i, _, a| Complex::new(amps[i], 0.0) * a.conj() / denomi[i])
            .transpose();

        let R = &G * X;

        let eig = R.symmetric_eigen();
        let e_arg = eig
            .eigenvectors
            .row(eig.eigenvalues.imax())
            .map(|e| e.argument());

        let sigma = MatrixXcf::from_diagonal(&VectorXcf::from_iterator(
            n,
            G.column_iter()
                .map(|col| {
                    col.iter()
                        .zip(amps.iter())
                        .map(|(a, &amp)| a.abs() * amp)
                        .sum()
                })
                .map(|s: Float| Complex::new((s / m as Float).sqrt().pow(self.gamma), 0.0)),
        ));

        let g = append_matrix_row(G, &sigma);
        let f = VectorXcf::from_iterator(
            m + n,
            amps.iter()
                .zip(e_arg.iter())
                .map(|(amp, &e)| amp * (Complex::new(0., e)).exp())
                .chain((0..n).map(|_| Complex::new(0., 0.))),
        );

        let gt = g.adjoint();
        let gtg = &gt * g;
        let gtf = gt * f;
        let q = gtg.qr().solve(&gtf).unwrap();

        let max_coeff = q.camax();
        for (d, qe) in data.iter_mut().flatten().zip(q.iter()) {
            let duty = if self.normalize {
                0xFF00
            } else {
                let amp = qe.abs() / max_coeff;
                ((255.0 * amp) as u16) << 8
            };

            let phase = (qe.argument() + PI) / (2.0 * PI);
            let phase = (255.0 * (1.0 - phase)) as u16;

            *d = duty | phase;
        }
    }
}
