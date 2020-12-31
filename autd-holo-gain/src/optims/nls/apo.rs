/*
 * File: apo.rs
 * Project: nls
 * Created Date: 03/10/2020
 * Author: Shun Suzuki
 * -----
 * Last Modified: 31/12/2020
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2020 Hapis Lab. All rights reserved.
 *
 */

use autd::{
    consts::{DataArray, NUM_TRANS_IN_UNIT},
    geometry::Geometry,
    prelude::Vector3,
    Float, PI,
};
use na::ComplexField;

use crate::{
    macros::{generate_propagation_matrix, Complex, MatrixXcf, VectorXcf, VectorXf},
    Optimizer,
};

const EPS: Float = 1e-8;
const K_MAX: usize = 200;
const LAMBDA: Float = 2.0;
const LINE_SEARCH_MAX: usize = 100;

/// References
/// * Keisuke Hasegawa, Hiroyuki Shinoda, and Takaaki Nara. Volumetric acoustic holography andits application to self-positioning by single channel measurement.Journal of Applied Physics,127(24):244904, 2020.7
pub struct APO {
    pub eps: Float,
    pub k_max: usize,
    pub line_search_max: usize,
    pub lambda: Float,
    pub normalize: bool,
}

impl Default for APO {
    fn default() -> Self {
        Self {
            eps: EPS,
            k_max: K_MAX,
            line_search_max: LINE_SEARCH_MAX,
            lambda: LAMBDA,
            normalize: true,
        }
    }
}

impl APO {
    #[allow(non_snake_case)]
    fn make_Ri(G: &MatrixXcf, i: usize, m: usize) -> MatrixXcf {
        let mut Di = MatrixXcf::zeros(m, m);
        Di[(i, i)] = Complex::new(1., 0.);
        G.adjoint() * Di * G
    }

    #[allow(non_snake_case)]
    fn calc_J(p2: &VectorXf, q: &VectorXcf, Ris: &[MatrixXcf], m: usize, lambda: Float) -> Float {
        (0..m)
            .map(|i| {
                let s = (q.adjoint() * &Ris[i] * q)[0] - p2[i];
                s.modulus_squared()
            })
            .sum::<Float>()
            + q.dot(&q).abs() * lambda
    }

    #[allow(non_snake_case)]
    fn calc_nabla_J(
        p2: &VectorXf,
        q: &VectorXcf,
        Ris: &[MatrixXcf],
        m: usize,
        lambda: Float,
    ) -> VectorXcf {
        (0..m)
            .map(|i| {
                let s = p2[i] - (q.adjoint() * &Ris[i] * q)[0].abs();
                (&Ris[i] * q).scale(s)
            })
            .sum::<VectorXcf>()
            + q.scale(lambda)
    }

    // Does not consider Wolfe-Powell condition
    // Only search alpha in [0,1)
    #[allow(non_snake_case)]
    #[allow(clippy::many_single_char_names)]
    fn line_search(
        q: &VectorXcf,
        d: &VectorXcf,
        p2: &VectorXf,
        Ris: &[MatrixXcf],
        m: usize,
        lambda: Float,
        line_search_max: usize,
    ) -> Float {
        let mut alpha = 0.;
        let mut min = Float::INFINITY;

        for i in 0..line_search_max {
            let a = i as Float / line_search_max as Float;
            let v = Self::calc_J(p2, &(q + d.scale(a)), Ris, m, lambda);
            if v < min {
                alpha = a;
                min = v;
            }
        }

        alpha
    }
}

impl Optimizer for APO {
    #[allow(non_snake_case, clippy::many_single_char_names)]
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

        let p = VectorXcf::from_iterator(m, amps.iter().map(|&a| Complex::new(a, 0.)));
        let p2 = p.map(|v| v.modulus_squared());

        let I = MatrixXcf::identity(n, n);
        let q0 = (G.adjoint() * &G + I.scale(self.lambda))
            .qr()
            .solve(&(G.adjoint() * &p))
            .unwrap();
        let Ris: Vec<_> = (0..m).map(|i| Self::make_Ri(&G, i, m)).collect();

        let mut H = I;
        let mut q = q0;

        let mut nabla_J = Self::calc_nabla_J(&p2, &q, &Ris, m, self.lambda);
        for _ in 0..self.k_max {
            let d = -(&H * &nabla_J);

            let alpha = Self::line_search(&q, &d, &p2, &Ris, m, self.lambda, self.line_search_max);

            let d = d.scale(alpha);

            if d.norm() < self.eps {
                break;
            }

            let q_new = &q + &d;
            let nabla_J_new = Self::calc_nabla_J(&p2, &q_new, &Ris, m, self.lambda);

            let s = &nabla_J_new - nabla_J;
            let y = d;

            H = &H + &y * y.transpose() / y.dot(&s)
                - (&H * &s * s.transpose() * H.transpose()) / ((s.transpose() * &H * s)[0]);

            q = q_new;
            nabla_J = nabla_J_new;
        }

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
