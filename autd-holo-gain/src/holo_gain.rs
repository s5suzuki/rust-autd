/*
 * File: holo_gain.rs
 * Project: gain
 * Created Date: 22/11/2019
 * Author: Shun Suzuki
 * -----
 * Last Modified: 07/08/2020
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2019 Hapis Lab. All rights reserved.
 * -----
 * The following algorithm is originally developed by Seki Inoue et al.
 * S. Inoue et al, "Active Touch Perception Produced by Airborne Ultrasonic Haptic Hologram," Proc. 2015 IEEE World Haptics Conference, pp.362-367, Northwestern University, Evanston, II, USA, June 22–26, 2015.
 *
 */

use std::f64::consts::PI;

use autd::consts::{NUM_TRANS_IN_UNIT, ULTRASOUND_WAVELENGTH};
use autd::gain::adjust_amp;
use autd::gain::Gain;
use autd::geometry::{Geometry, Vector3};
use autd::utils::directivity_t4010a1 as dir;
use na::{ComplexField, Dynamic, Matrix, VecStorage, U1};
use rand::{thread_rng, Rng};

type Cf64 = na::Complex<f64>;
type MatrixXcf = Matrix<Cf64, Dynamic, Dynamic, VecStorage<Cf64, Dynamic, Dynamic>>;
type VectorXcf = Matrix<Cf64, Dynamic, U1, VecStorage<Cf64, Dynamic, U1>>;

const REPEAT_SDP: usize = 10;
const LAMBDA_SDP: f64 = 0.8;

pub struct HoloGain {
    foci: Vec<Vector3>,
    amps: Vec<f64>,
    data: Option<Vec<u8>>,
}

impl HoloGain {
    pub fn create(foci: Vec<Vector3>, amps: Vec<f64>) -> Box<HoloGain> {
        assert_eq!(foci.len(), amps.len());
        Box::new(HoloGain {
            foci,
            amps,
            data: None,
        })
    }
}

impl HoloGain {
    fn transfer(trans_pos: Vector3, trans_norm: Vector3, target_pos: Vector3) -> Cf64 {
        let diff = target_pos - trans_pos;
        let dist = diff.norm();
        let theta = diff.angle(&trans_norm) * 180.0 / PI;
        let directivity = dir(theta);

        directivity / dist
            * (Cf64::new(-dist * 1.15e-4, -2. * PI / ULTRASOUND_WAVELENGTH * dist)).exp()
    }
}

impl Gain for HoloGain {
    fn get_data(&self) -> &[u8] {
        assert!(self.data.is_some());
        match &self.data {
            Some(data) => data,
            None => panic!(),
        }
    }

    #[allow(clippy::many_single_char_names)]
    fn build(&mut self, geometry: &Geometry) {
        if self.data.is_some() {
            return;
        }

        let ndevice = geometry.num_devices();
        let ntrans = NUM_TRANS_IN_UNIT * ndevice;
        let mut data = Vec::with_capacity(ntrans * 2);
        let foci = &self.foci;
        let amps = &self.amps;

        let alpha = 1e-3;
        let m = foci.len();
        let n = ntrans;
        let mut b = MatrixXcf::from_vec(m, n, vec![Cf64::new(0., 0.); m * n]);
        let mut p = MatrixXcf::from_vec(m, m, vec![Cf64::new(0., 0.); m * m]);

        let mut rng = thread_rng();

        for i in 0..m {
            p[(i, i)] = Cf64::new(amps[i], 0.);
            let tp = foci[i];
            for j in 0..n {
                b[(i, j)] = HoloGain::transfer(geometry.position(j), geometry.direction(j), tp);
            }
        }
        let svd = b.clone().svd(true, true);
        let mut singular_velues_inv = svd.singular_values.clone();
        for i in 0..singular_velues_inv.len() {
            singular_velues_inv[i] = singular_velues_inv[i]
                / (singular_velues_inv[i] * singular_velues_inv[i] + alpha * alpha);
        }
        let mut singular_velues_inv_mat = MatrixXcf::from_vec(m, m, vec![Cf64::new(0., 0.); m * m]);
        singular_velues_inv_mat.set_diagonal(&singular_velues_inv.map(|r| Cf64::new(r, 0.)));
        let pinv_b = match (&svd.v_t, &svd.u) {
            (Some(v_t), Some(u)) => v_t.adjoint() * singular_velues_inv_mat * u.adjoint(),
            _ => unreachable!(),
        };
        let mm = &p * (MatrixXcf::identity(m, m) - b * &pinv_b) * &p;
        let mut x = MatrixXcf::identity(m, m);

        for _ in 0..(m * REPEAT_SDP) {
            let ii = (m as f64 * rng.gen_range(0., 1.)) as usize;
            let xc = x.clone().remove_row(ii).remove_column(ii);
            let mmc = mm.column(ii).remove_row(ii);
            let xb = xc * &mmc;
            let gamma = xb.adjoint() * mmc;
            let gamma = gamma[(0, 0)];
            if gamma.re > 0. {
                let xb = xb.scale(-(LAMBDA_SDP / gamma.re).sqrt());
                x.slice_mut((ii, 0), (1, ii))
                    .copy_from(&xb.slice((0, 0), (ii, 1)).adjoint());
                x.slice_mut((ii, ii + 1), (1, m - ii - 1))
                    .copy_from(&xb.slice((ii, 0), (m - 1 - ii, 1)).adjoint());
                x.slice_mut((0, ii), (ii, 1))
                    .copy_from(&xb.slice((0, 0), (ii, 1)));
                x.slice_mut((ii + 1, ii), (m - ii - 1, 1))
                    .copy_from(&xb.slice((ii, 0), (m - 1 - ii, 1)));
            } else {
                let z1 = VectorXcf::from_vec(vec![Cf64::new(0., 0.,); ii]);
                let z2 = VectorXcf::from_vec(vec![Cf64::new(0., 0.,); m - ii - 1]);
                x.slice_mut((ii, 0), (1, ii)).copy_from(&z1.adjoint());
                x.slice_mut((ii, ii + 1), (1, m - ii - 1))
                    .copy_from(&z2.adjoint());
                x.slice_mut((0, ii), (ii, 1)).copy_from(&z1);
                x.slice_mut((ii + 1, ii), (m - ii - 1, 1)).copy_from(&z2);
            }
        }

        let ces = na::SymmetricEigen::new(x);
        let evs = ces.eigenvalues;
        let mut abseiv = 0.;
        let mut idx = 0;
        for j in 0..evs.len() {
            let eiv = evs[j].abs();
            if abseiv < eiv {
                abseiv = eiv;
                idx = j;
            }
        }

        let u = ces.eigenvectors.column(idx);
        let q = pinv_b * p * u;
        //auto maxCoeff = sqrt(q.cwiseAbs2().maxCoeff());
        for j in 0..n {
            let famp = 1.; //abs(q(j)) / maxCoeff;
            let fphase = q[j].arg() / (2.0 * PI) + 0.5;
            let amp = (famp * 255.) as u8;
            let phase = ((1. - fphase) * 255.) as u8;
            let d = adjust_amp(amp);
            let s = phase;
            data.push(s);
            data.push(d);
        }

        self.data = Some(data);
    }
}
