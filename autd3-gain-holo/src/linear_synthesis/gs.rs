/*
 * File: gs.rs
 * Project: linear_synthesis
 * Created Date: 29/05/2021
 * Author: Shun Suzuki
 * -----
 * Last Modified: 31/05/2022
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2021 Shun Suzuki. All rights reserved.
 *
 */

use crate::{macros::generate_propagation_matrix, Backend, Complex, Transpose, VectorXc};
use anyhow::Result;
use autd3_core::{
    gain::{Gain, GainProps, IGain},
    geometry::{DriveData, Geometry, Transducer, Vector3},
    NUM_TRANS_IN_UNIT,
};
use autd3_traits::Gain;
use nalgebra::ComplexField;
use std::{f64::consts::PI, marker::PhantomData};

/// Reference
/// * Asier Marzo and Bruce W Drinkwater. Holographic acoustic tweezers.Proceedings of theNational Academy of Sciences, 116(1):84–89, 2019.
#[derive(Gain)]
pub struct GS<B: Backend, T: Transducer> {
    props: GainProps<T>,
    foci: Vec<Vector3>,
    amps: Vec<f64>,
    repeat: usize,
    backend: PhantomData<B>,
}

impl<B: Backend, T: Transducer> GS<B, T> {
    pub fn new(foci: Vec<Vector3>, amps: Vec<f64>) -> Self {
        Self::with_param(foci, amps, 100)
    }

    pub fn with_param(foci: Vec<Vector3>, amps: Vec<f64>, repeat: usize) -> Self {
        assert!(foci.len() == amps.len());
        Self {
            props: GainProps::default(),
            foci,
            amps,
            repeat,
            backend: PhantomData,
        }
    }
}

impl<B: Backend, T: Transducer> IGain<T> for GS<B, T> {
    fn calc(&mut self, geometry: &Geometry<T>) -> Result<()> {
        let m = self.foci.len();
        let n = geometry.num_devices() * NUM_TRANS_IN_UNIT;

        let g = generate_propagation_matrix(geometry, &self.foci);

        let q0 = VectorXc::from_element(n, Complex::new(1., 0.));
        let mut q = q0.clone();

        let mut gamma = VectorXc::zeros(m);
        let mut p = VectorXc::zeros(m);
        let mut xi = VectorXc::zeros(n);
        for _ in 0..self.repeat {
            B::matrix_mul_vec(
                Transpose::NoTrans,
                Complex::new(1., 0.),
                &g,
                &q,
                Complex::new(0., 0.),
                &mut gamma,
            );
            for i in 0..m {
                p[i] = gamma[i] / gamma[i].abs() * self.amps[i];
            }
            B::matrix_mul_vec(
                Transpose::ConjTrans,
                Complex::new(1., 0.),
                &g,
                &p,
                Complex::new(0., 0.),
                &mut xi,
            );
            for i in 0..n {
                q[i] = xi[i] / xi[i].abs() * q0[i];
            }
        }

        geometry.transducers().for_each(|tr| {
            let phase = q[tr.id()].argument() / (2.0 * PI) + 0.5;
            self.props.drives.set_drive(tr, phase, 1.0);
        });

        Ok(())
    }
}
