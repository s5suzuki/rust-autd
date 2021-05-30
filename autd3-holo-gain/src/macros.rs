/*
 * File: macros.rs
 * Project: src
 * Created Date: 28/05/2021
 * Author: Shun Suzuki
 * -----
 * Last Modified: 28/05/2021
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2021 Hapis Lab. All rights reserved.
 *
 */

use crate::{Complex, MatrixXc, VectorXc};
use autd3_core::{
    geometry::{Geometry, Vector3},
    hardware_defined::{DataArray, NUM_TRANS_IN_UNIT},
    utils::{adjust_amp, directivity_t4010a1 as directivity},
};
use nalgebra::ComplexField;
use std::f64::consts::PI;

pub fn propagate(
    source_pos: Vector3,
    source_dir: Vector3,
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
        iproduct!(0..num_device, 0..NUM_TRANS_IN_UNIT)
            .map(|(dev, i)| {
                foci.iter()
                    .map(|&fp| {
                        propagate(
                            geometry.position_by_local_idx(dev, i),
                            geometry.direction(dev),
                            geometry.attenuation,
                            wavenum,
                            fp,
                        )
                    })
                    .collect::<Vec<_>>()
            })
            .flatten(),
    )
}

pub fn set_from_complex_drive(
    data: &mut [DataArray],
    drive: &VectorXc,
    normalize: bool,
    max_coefficient: f64,
) {
    let n = drive.len();
    let mut dev_idx = 0;
    let mut trans_idx = 0;
    for j in 0..n {
        let f_amp = if normalize {
            1.0
        } else {
            drive[j].abs() / max_coefficient
        };
        let f_phase = drive[j].argument() / (2.0 * PI) + 0.5;
        let phase = ((1.0 - f_phase) * 255.0) as u16;
        let duty = ((adjust_amp(f_amp) as u16) << 8) & 0xFF00;
        data[dev_idx][trans_idx] = duty | phase;
        trans_idx += 1;
        if trans_idx == NUM_TRANS_IN_UNIT {
            dev_idx += 1;
            trans_idx = 0;
        }
    }
}
