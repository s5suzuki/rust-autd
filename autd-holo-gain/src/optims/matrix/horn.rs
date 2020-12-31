/*
 * File: horn.rs
 * Project: multiple_foci
 * Created Date: 27/05/2020
 * Author: Shun Suzuki
 * -----
 * Last Modified: 31/12/2020
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2020 Hapis Lab. All rights reserved.
 *
 */

use crate::optims::{macros::*, Optimizer};
use autd::{consts::DataArray, geometry::Geometry, prelude::Vector3, Float, PI};
use na::ComplexField;
use rand::{thread_rng, Rng};

const REPEAT_SDP: usize = 100;
const LAMBDA_SDP: Float = 0.8;
const TIKHONOV_DEFAULT: Float = 1e-5;

/// Reference
/// * Inoue, Seki, Yasutoshi Makino, and Hiroyuki Shinoda. "Active touch perception produced by airborne ultrasonic haptic hologram." 2015 IEEE World Haptics Conference (WHC). IEEE, 2015.
pub struct Horn {
    pub repeat: usize,
    pub lambda: Float,
    pub tikhonov_parameter: Float,
    pub normalize: bool,
}

impl Horn {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for Horn {
    fn default() -> Self {
        Self {
            repeat: REPEAT_SDP,
            lambda: LAMBDA_SDP,
            tikhonov_parameter: TIKHONOV_DEFAULT,
            normalize: true,
        }
    }
}

impl Optimizer for Horn {
    #[allow(clippy::many_single_char_names)]
    #[allow(non_snake_case)]
    fn optimize(
        &self,
        geometry: &Geometry,
        foci: &[Vector3],
        amps: &[Float],
        atten: Float,
        data: &mut [DataArray],
    ) {
        let m = foci.len();

        let G = generate_propagation_matrix(geometry, atten, foci);
        let P = MatrixXcf::from_diagonal(&VectorXcf::from_iterator(
            m,
            amps.iter().map(|&a| Complex::new(a, 0.)),
        ));

        let G_pinv = pseudo_inverse_with_reg(&G, self.tikhonov_parameter);
        let MM = &P * (MatrixXcf::identity(m, m) - G * &G_pinv) * &P;
        let mut X = MatrixXcf::identity(m, m);

        let mut rng = thread_rng();
        let lambda = self.lambda;
        for _ in 0..(m * self.repeat) {
            let ii = (m as Float * rng.gen_range(0.0..1.0)) as usize;
            let Xc = X.clone().remove_row(ii).remove_column(ii);
            let MMc = MM.column(ii).remove_row(ii);
            let Xb = Xc * &MMc;
            let gamma = (Xb.adjoint() * MMc)[(0, 0)];
            if gamma.re > 0. {
                let Xb = Xb.scale(-(lambda / gamma.re).sqrt());
                X.slice_mut((ii, 0), (1, ii))
                    .copy_from(&Xb.slice((0, 0), (ii, 1)).adjoint());
                X.slice_mut((ii, ii + 1), (1, m - ii - 1))
                    .copy_from(&Xb.slice((ii, 0), (m - 1 - ii, 1)).adjoint());
                X.slice_mut((0, ii), (ii, 1))
                    .copy_from(&Xb.slice((0, 0), (ii, 1)));
                X.slice_mut((ii + 1, ii), (m - ii - 1, 1))
                    .copy_from(&Xb.slice((ii, 0), (m - 1 - ii, 1)));
            } else {
                let z1 = VectorXcf::zeros(ii);
                let z2 = VectorXcf::zeros(m - ii - 1);
                X.slice_mut((ii, 0), (1, ii)).copy_from(&z1.adjoint());
                X.slice_mut((ii, ii + 1), (1, m - ii - 1))
                    .copy_from(&z2.adjoint());
                X.slice_mut((0, ii), (ii, 1)).copy_from(&z1);
                X.slice_mut((ii + 1, ii), (m - ii - 1, 1)).copy_from(&z2);
            }
        }

        let eig = na::SymmetricEigen::new(X);
        let u = eig.eigenvectors.column(eig.eigenvalues.imax());
        let q = G_pinv * P * u;
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
