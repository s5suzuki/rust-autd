/*
 * File: greedy.rs
 * Project: combinational
 * Created Date: 03/06/2021
 * Author: Shun Suzuki
 * -----
 * Last Modified: 24/11/2021
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2021 Hapis Lab. All rights reserved.
 *
 */

use std::f64::consts::PI;

use crate::{macros::propagate, Complex};
use anyhow::Result;
use autd3_core::{
    gain::{Gain, GainData},
    geometry::{Geometry, Vector3},
};
use autd3_traits::Gain;
use nalgebra::ComplexField;

/// Reference
/// * Shun Suzuki, Masahiro Fujiwara, Yasutoshi Makino, and Hiroyuki Shinoda, “Radiation Pressure Field Reconstruction for Ultrasound Midair Haptics by Greedy Algorithm with Brute-Force Search,” in IEEE Transactions on Haptics, doi: 10.1109/TOH.2021.3076489
#[derive(Gain)]
pub struct Greedy {
    data: Vec<GainData>,
    built: bool,
    foci: Vec<Vector3>,
    amps: Vec<f64>,
    phases: Vec<Complex>,
}

impl Greedy {
    pub fn new(foci: Vec<Vector3>, amps: Vec<f64>) -> Self {
        Self::with_param(foci, amps, 16)
    }

    pub fn with_param(foci: Vec<Vector3>, amps: Vec<f64>, phase_div: usize) -> Self {
        assert!(foci.len() == amps.len());
        let mut phases = Vec::with_capacity(phase_div);
        for i in 0..phase_div {
            phases.push(Complex::new(0., 2.0 * PI * i as f64 / phase_div as f64).exp());
        }
        Self {
            data: vec![],
            built: false,
            foci,
            amps,
            phases,
        }
    }

    #[allow(clippy::many_single_char_names)]
    #[allow(clippy::unnecessary_wraps)]
    fn calc(&mut self, geometry: &Geometry) -> Result<()> {
        let m = self.foci.len();

        let wave_num = 2.0 * PI / geometry.wavelength;
        let attenuation = geometry.attenuation;

        let mut tmp = Vec::with_capacity(self.phases.len());
        tmp.resize(self.phases.len(), vec![Complex::new(0., 0.); m]);

        let mut cache = Vec::with_capacity(m);
        cache.resize(m, Complex::new(0., 0.));

        fn transfer_foci(
            trans_pos: Vector3,
            trans_dir: Vector3,
            phase: Complex,
            wave_num: f64,
            atten: f64,
            foci: &[Vector3],
            res: &mut [Complex],
        ) {
            for i in 0..foci.len() {
                res[i] = propagate(trans_pos, trans_dir, atten, wave_num, foci[i]) * phase;
            }
        }

        for (dev, data) in geometry.devices().zip(self.data.iter_mut()) {
            let trans_dir = dev.z_direction();
            for (&trans, d) in dev.transducers().zip(data.iter_mut()) {
                let mut min_idx = 0;
                let mut min_v = std::f64::INFINITY;
                for (idx, &phase) in self.phases.iter().enumerate() {
                    transfer_foci(
                        trans,
                        trans_dir,
                        phase,
                        wave_num,
                        attenuation,
                        &self.foci,
                        &mut tmp[idx],
                    );
                    let mut v = 0.0;
                    for (j, c) in cache.iter().enumerate() {
                        v += (self.amps[j] - (tmp[idx][j] + c).abs()).abs();
                    }

                    if v < min_v {
                        min_v = v;
                        min_idx = idx;
                    }
                }

                for (j, c) in cache.iter_mut().enumerate() {
                    *c += tmp[min_idx][j];
                }

                const DUTY: u8 = 0xFF;
                d.duty = DUTY;
                d.phase = autd3_core::utils::to_phase(self.phases[min_idx].argument());
            }
        }
        Ok(())
    }
}
