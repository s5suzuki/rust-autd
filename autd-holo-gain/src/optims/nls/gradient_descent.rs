/*
 * File: gradient_descent.rs
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

use autd::{
    consts::{DataArray, NUM_TRANS_IN_UNIT},
    geometry::Geometry,
    prelude::Vector3,
    Float, PI,
};

use super::macros::*;
use crate::{macros::VectorXf, Optimizer};

const EPS: Float = 1e-6;
const K_MAX: usize = 10_000;

/// Gauss-Newton
pub struct GradientDescent {
    pub eps: Float,
    pub k_max: usize,
}

impl GradientDescent {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for GradientDescent {
    fn default() -> Self {
        Self {
            eps: EPS,
            k_max: K_MAX,
        }
    }
}

impl Optimizer for GradientDescent {
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
        for _ in 0..self.k_max {
            let T = make_T(&x, n, m);
            let Jtf = calc_Jtf(&BhB, &T);
            if Jtf.max() <= self.eps {
                break;
            }
            x = &x - &(0.1 * Jtf);
        }

        let duty = 0xFF00;
        for (d, xe) in data.iter_mut().flatten().zip(x.iter()) {
            let phase = (xe % (2.0 * PI)) / (2.0 * PI);
            let phase = (255.0 * (1.0 - phase)) as u16;

            *d = duty | phase;
        }
    }
}
