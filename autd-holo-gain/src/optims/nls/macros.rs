/*
 * File: macros.rs
 * Project: nls
 * Created Date: 18/11/2020
 * Author: Shun Suzuki
 * -----
 * Last Modified: 31/12/2020
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2020 Hapis Lab. All rights reserved.
 *
 */

use autd::{geometry::Geometry, prelude::Vector3, Float};

use crate::macros::{
    append_matrix_col, generate_propagation_matrix, Complex, MatrixXcf, MatrixXf, VectorXcf,
    VectorXf,
};

#[allow(non_snake_case)]
pub fn make_BhB(
    geometry: &Geometry,
    atten: Float,
    amps: &[Float],
    foci: &[Vector3],
    m: usize,
) -> MatrixXcf {
    let P = MatrixXcf::from_diagonal(&VectorXcf::from_iterator(
        m,
        amps.iter().map(|a| Complex::new(-a, 0.)),
    ));
    let G = generate_propagation_matrix(geometry, atten, foci);

    let B = append_matrix_col(G, &P);
    B.adjoint() * B
}

#[allow(non_snake_case)]
pub fn make_T(x: &VectorXf, n: usize, m: usize) -> VectorXcf {
    VectorXcf::from_iterator(n + m, x.iter().map(|x| Complex::new(0., -x).exp()))
}

#[allow(non_snake_case)]
pub fn calc_Jtf(BhB: &MatrixXcf, T: &VectorXcf) -> VectorXf {
    let TTh = T * T.adjoint();
    let BhB_TTh = BhB.component_mul(&TTh);
    BhB_TTh.map(|c| c.im).column_sum()
}

#[allow(non_snake_case)]
pub fn calc_JtJ_Jtf(BhB: &MatrixXcf, T: &VectorXcf) -> (MatrixXf, VectorXf) {
    let TTh = T * T.adjoint();
    let BhB_TTh = BhB.component_mul(&TTh);
    let JtJ = BhB_TTh.map(|c| c.re);
    let Jtf = BhB_TTh.map(|c| c.im).column_sum();
    (JtJ, Jtf)
}

#[allow(non_snake_case)]
pub fn calc_Fx(BhB: &MatrixXcf, x: &VectorXf, n: usize, m: usize) -> Float {
    let t = VectorXcf::from_iterator(n + m, x.iter().map(|&x| Complex::new(0., x).exp()));
    (t.adjoint() * BhB * t)[(0, 0)].re
}
