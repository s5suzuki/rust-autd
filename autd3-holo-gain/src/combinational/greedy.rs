/*
 * File: greedy.rs
 * Project: combinational
 * Created Date: 03/06/2021
 * Author: Shun Suzuki
 * -----
 * Last Modified: 05/06/2021
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2021 Hapis Lab. All rights reserved.
 *
 */

use crate::{macros::propagate, Complex};
use anyhow::Result;
use autd3_core::{
    gain::Gain,
    geometry::{Geometry, Vector3},
    hardware_defined::{DataArray, NUM_TRANS_IN_UNIT},
};
use autd3_traits::Gain;
use nalgebra::ComplexField;

/// Reference
/// * Shun Suzuki, Masahiro Fujiwara, Yasutoshi Makino, and Hiroyuki Shinoda, “Radiation Pressure Field Reconstruction for Ultrasound Midair Haptics by Greedy Algorithm with Brute-Force Search,” in IEEE Transactions on Haptics, doi: 10.1109/TOH.2021.3076489
#[derive(Gain)]
pub struct Greedy {
    data: Vec<DataArray>,
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
            phases.push(
                Complex::new(0., 2.0 * std::f64::consts::PI * i as f64 / phase_div as f64).exp(),
            );
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

        let wave_num = 2.0 * std::f64::consts::PI / geometry.wavelength;
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

        for dev in 0..geometry.num_devices() {
            for i in 0..NUM_TRANS_IN_UNIT {
                let trans_pos = geometry.position_by_local_idx(dev, i);
                let trans_dir = geometry.direction(dev);
                let mut min_idx = 0;
                let mut min_v = std::f64::INFINITY;
                for (idx, &phase) in self.phases.iter().enumerate() {
                    transfer_foci(
                        trans_pos,
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

                const DUTY: u16 = 0xFF00;
                let phase = ((1.0
                    - (self.phases[min_idx].argument() + std::f64::consts::PI)
                        / (2.0 * std::f64::consts::PI))
                    * 255.0) as u16;
                self.data[dev][i] = DUTY | phase;
            }
        }

        self.built = true;
        Ok(())
    }
}
