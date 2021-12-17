/*
 * File: macros.rs
 * Project: src
 * Created Date: 28/05/2021
 * Author: Shun Suzuki
 * -----
 * Last Modified: 16/12/2021
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2021 Hapis Lab. All rights reserved.
 *
 */

use crate::{Complex, MatrixXc, VectorXc};
use autd3_core::{
    geometry::{Geometry, Vector3},
    hardware_defined::{Drive, NUM_TRANS_IN_UNIT},
    utils::{directivity_t4010a1 as directivity, to_duty},
};
use nalgebra::ComplexField;
use std::f64::consts::PI;

pub fn propagate(
    source_pos: &Vector3,
    source_dir: &Vector3,
    atten: f64,
    wavenum: f64,
    target: Vector3,
) -> Complex {
    let diff = target - source_pos;
    let dist = diff.norm();
    let theta = source_dir.angle(&diff);
    let d = directivity(theta);
    let r = d * (-dist * atten).exp() / dist;
    let phi = -wavenum * dist;
    r * Complex::new(0., phi).exp()
}

pub fn generate_propagation_matrix(geometry: &Geometry, foci: &[Vector3]) -> MatrixXc {
    let m = foci.len();
    let num_device = geometry.num_devices();
    let num_trans = num_device * NUM_TRANS_IN_UNIT;

    let wavenum = 2. * PI / geometry.wavelength;

    MatrixXc::from_iterator(
        m,
        num_trans,
        geometry.transducers().flat_map(|trans| {
            foci.iter().map(move |&fp| {
                propagate(
                    trans.position(),
                    trans.z_direction(),
                    geometry.attenuation,
                    wavenum,
                    fp,
                )
            })
        }),
    )
}

pub fn set_from_complex_drive(
    data: &mut [Drive],
    drive: &VectorXc,
    normalize: bool,
    max_coefficient: f64,
) {
    let n = drive.len();
    let _dev_idx = 0;
    let _trans_idx = 0;
    for j in 0..n {
        let f_amp = if normalize {
            1.0
        } else {
            drive[j].abs() / max_coefficient
        };
        data[j].duty = to_duty(f_amp);
        data[j].phase = autd3_core::utils::to_phase(drive[j].argument());
    }
}
