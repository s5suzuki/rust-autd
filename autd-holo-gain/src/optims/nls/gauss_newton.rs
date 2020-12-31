/*
 * File: gauss_newton.rs
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

use super::macros::*;
use crate::{macros::VectorXf, Optimizer};

const EPS_1: Float = 1e-8;
const EPS_2: Float = 1e-8;
const K_MAX: usize = 200;

/// Gauss-Newton
pub struct GaussNewton {
    pub eps_1: Float,
    pub eps_2: Float,
    pub k_max: usize,
}

impl GaussNewton {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for GaussNewton {
    fn default() -> Self {
        Self {
            eps_1: EPS_1,
            eps_2: EPS_2,
            k_max: K_MAX,
        }
    }
}

impl Optimizer for GaussNewton {
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

        let n_param = n + m;

        let x0 = VectorXf::zeros(n_param);

        let BhB = make_BhB(geometry, atten, amps, foci, m);

        let mut x = x0;

        let T = make_T(&x, n, m);
        let (mut A, mut g) = calc_JtJ_Jtf(&BhB, &T);

        let mut found = g.max() <= self.eps_1;
        for _ in 0..self.k_max {
            if found {
                break;
            }

            let h_lm = match A.clone().qr().solve(&g) {
                Some(v) => -v,
                None => {
                    break;
                }
            };

            // let h_lm = match A.clone().pseudo_inverse(1e-3) {
            //     Ok(Ai) => -(Ai * &g),
            //     Err(_) => {
            //         break;
            //     }
            // };

            if h_lm.norm() <= self.eps_2 * (x.norm() + self.eps_2) {
                found = true;
            } else {
                x = &x + &h_lm;
                let T = make_T(&x, n, m);
                let (A_new, g_new) = calc_JtJ_Jtf(&BhB, &T);
                A = A_new;
                g = g_new;
                found = g.max() <= self.eps_1;
            }
        }

        let duty = 0xFF00;
        for (d, xe) in data.iter_mut().flatten().zip(x.iter()) {
            let phase = (xe % (2.0 * PI)) / (2.0 * PI);
            let phase = (255.0 * (1.0 - phase)) as u16;

            *d = duty | phase;
        }
    }
}
