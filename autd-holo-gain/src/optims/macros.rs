/*
 * File: macros.rs
 * Project: multiple_foci
 * Created Date: 18/11/2020
 * Author: Shun Suzuki
 * -----
 * Last Modified: 08/03/2021
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2020 Hapis Lab. All rights reserved.
 *
 */

use autd::{consts::NUM_TRANS_IN_UNIT, geometry::Geometry, utils::directivity_t4010a1 as dir, PI};
use autd::{prelude::Vector3, Float};
use na::{ComplexField, Dynamic, Matrix, VecStorage, U1};

pub type Complex = na::Complex<Float>;
pub type MatrixXcf = Matrix<Complex, Dynamic, Dynamic, VecStorage<Complex, Dynamic, Dynamic>>;
pub type VectorXcf = Matrix<Complex, Dynamic, U1, VecStorage<Complex, Dynamic, U1>>;
pub type MatrixXf = Matrix<Float, Dynamic, Dynamic, VecStorage<Float, Dynamic, Dynamic>>;
pub type VectorXf = Matrix<Float, Dynamic, U1, VecStorage<Float, Dynamic, U1>>;

pub fn propagate(
    source_pos: Vector3,
    source_dir: Vector3,
    atten: Float,
    wavenum: Float,
    target: Vector3,
) -> Complex {
    let diff = target - source_pos;
    let dist = diff.norm();
    let theta = source_dir.angle(&diff);
    let d = dir(theta);
    let r = d * (-dist * atten).exp() / dist;
    let phi = -wavenum * dist;
    r * Complex::new(0., phi).exp()
}

pub fn generate_propagation_matrix(
    geometry: &Geometry,
    atten: Float,
    foci: &[Vector3],
) -> MatrixXcf {
    let m = foci.len();
    let num_device = geometry.num_devices();
    let num_trans = num_device * NUM_TRANS_IN_UNIT;

    let wavenum = 2. * PI / geometry.wavelength();

    MatrixXcf::from_iterator(
        m,
        num_trans,
        iproduct!(0..num_device, 0..NUM_TRANS_IN_UNIT)
            .map(|(dev, i)| {
                foci.iter()
                    .map(|&fp| {
                        propagate(
                            geometry.position_by_local_idx(dev, i),
                            geometry.direction(dev),
                            atten,
                            wavenum,
                            fp,
                        )
                    })
                    .collect::<Vec<_>>()
            })
            .flatten(),
    )
}

pub fn pseudo_inverse_with_reg(m: &MatrixXcf, alpha: Float) -> MatrixXcf {
    let svd = m.clone().svd(true, true);
    let s_inv = MatrixXcf::from_diagonal(
        &svd.singular_values
            .map(|s| Complex::new(s / (s * s + alpha * alpha), 0.)),
    );
    match (&svd.v_t, &svd.u) {
        (Some(v_t), Some(u)) => v_t.adjoint() * s_inv * u.adjoint(),
        _ => unreachable!(),
    }
}

pub fn append_matrix_row(to: MatrixXcf, src: &MatrixXcf) -> MatrixXcf {
    assert_eq!(to.ncols(), src.ncols());

    let new_cols = to.ncols();
    let to_rows = to.nrows();
    let new_rows = to.nrows() + src.nrows();

    let mut new_mat = to.resize(new_rows, new_cols, Default::default());
    new_mat
        .slice_mut((to_rows, 0), (src.nrows(), src.ncols()))
        .copy_from(src);

    new_mat
}

pub fn append_matrix_col(to: MatrixXcf, src: &MatrixXcf) -> MatrixXcf {
    assert_eq!(to.nrows(), src.nrows());

    let new_rows = to.nrows();
    let to_cols = to.ncols();
    let new_cols = to.ncols() + src.ncols();

    let mut new_mat = to.resize(new_rows, new_cols, Default::default());
    new_mat
        .slice_mut((0, to_cols), (src.nrows(), src.ncols()))
        .copy_from(src);

    new_mat
}
