/*
 * File: gs_pat.rs
 * Project: multiple_foci
 * Created Date: 01/10/2020
 * Author: Shun Suzuki
 * -----
 * Last Modified: 31/12/2020
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2020 Hapis Lab. All rights reserved.
 *
 */

use crate::{
    macros::{generate_propagation_matrix, Complex, VectorXcf},
    Optimizer,
};
use autd::{consts::DataArray, geometry::Geometry, prelude::Vector3, Float, PI};
use na::ComplexField;

const REPEAT: usize = 100;

/// Reference
/// * Diego Martinez Plasencia et al. "Gs-pat: high-speed multi-point sound-fields for phased arrays of transducers," ACMTrans-actions on Graphics (TOG), 39(4):138–1, 2020.
///
/// Not yet been implemented with GPU.
pub struct GSPAT {
    pub repeat: usize,
    pub normalize: bool,
}

impl GSPAT {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for GSPAT {
    fn default() -> Self {
        Self {
            repeat: REPEAT,
            normalize: true,
        }
    }
}

impl Optimizer for GSPAT {
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

        let G = generate_propagation_matrix(geometry, atten, foci);

        let denomi = G.map(|a| a.abs()).row_sum();
        let B = G
            .map_with_location(|_, j, a| a.conj() / (denomi[j] * denomi[j]))
            .transpose();

        let R = &G * &B;

        let p0 = VectorXcf::from_iterator(m, amps.iter().map(|&a| Complex::new(a, 0.0)));
        let mut p = p0.clone();
        let mut gamma = &R * p;

        for _ in 0..self.repeat {
            p = VectorXcf::from_iterator(
                m,
                gamma.iter().zip(p0.iter()).map(|(g, &p)| g / g.abs() * p),
            );
            gamma = &R * p;
        }
        p = VectorXcf::from_iterator(
            m,
            gamma
                .iter()
                .zip(p0.iter())
                .map(|(g, &p)| g / (g.abs() * g.abs()) * p * p),
        );

        let q = B * p;

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
