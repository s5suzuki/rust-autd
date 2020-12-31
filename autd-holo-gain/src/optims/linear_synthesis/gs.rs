/*
 * File: gs.rs
 * Project: multiple_foci
 * Created Date: 02/10/2020
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
use autd::{
    consts::{DataArray, NUM_TRANS_IN_UNIT},
    geometry::Geometry,
    prelude::Vector3,
    Float, PI,
};
use na::ComplexField;

const REPEAT: usize = 100;

/// Reference
/// * Asier Marzo and Bruce W Drinkwater. Holographic acoustic tweezers.Proceedings of theNational Academy of Sciences, 116(1):84–89, 2019.
pub struct GS {
    pub repeat: usize,
    pub normalize: bool,
}

impl GS {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for GS {
    fn default() -> Self {
        Self {
            repeat: REPEAT,
            normalize: true,
        }
    }
}

impl Optimizer for GS {
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
        let Gh = G.map(|a| a.conj()).transpose();

        let p0 = VectorXcf::from_iterator(m, amps.iter().map(|&a| Complex::new(a, 0.0)));
        let q0 = VectorXcf::from_iterator(n, (0..n).map(|_| Complex::new(1.0, 0.0)));

        let mut q = q0.clone();

        for _ in 0..self.repeat {
            let gamma = &G * q;
            let p = VectorXcf::from_iterator(
                m,
                gamma.iter().zip(p0.iter()).map(|(g, &p)| g / g.abs() * p),
            );

            let xi = &Gh * p;
            q = VectorXcf::from_iterator(
                n,
                xi.iter().zip(q0.iter()).map(|(x, &q)| x / x.abs() * q),
            );
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
