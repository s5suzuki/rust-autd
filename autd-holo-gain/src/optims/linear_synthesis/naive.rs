/*
 * File: naive.rs
 * Project: linear_synthesis
 * Created Date: 03/10/2020
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

/// Naive linear synthesis
pub struct Naive {
    pub normalize: bool,
}

impl Naive {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for Naive {
    fn default() -> Self {
        Self { normalize: true }
    }
}

impl Optimizer for Naive {
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

        let Gh = G.map(|a| a.conj()).transpose();
        let p = VectorXcf::from_iterator(m, amps.iter().map(|&a| Complex::new(a, 0.0)));
        let q = Gh * p;

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
