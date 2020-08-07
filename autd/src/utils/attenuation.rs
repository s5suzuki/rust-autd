/*
 * File: attenuation.rs
 * Project: common
 * Created Date: 14/05/2020
 * Author: Shun Suzuki
 * -----
 * Last Modified: 14/05/2020
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2020 Hapis Lab. All rights reserved.
 *
 */

/// Returns an attenuation coefficients due to atmospheric absorption in a unit of [Np/mm].
///
/// Bass, Henry E., et al. "Atmospheric absorption of sound: Further developments." The Journal of the Acoustical Society of America 97.1 (1995): 680-683.
///
/// # Arguments
///
/// * `freq` - A frequency of sound [Hz]
/// * `hr` - A relative humidity [%]
/// * `ps` - An atmospheric pressure [atm]
/// * `ps0` - A reference atmospheric pressure [atm]
/// * `t` - An atmospheric temperature [K]
/// * `t0` - A reference temperature [K]
/// * `t01` - A triple-point isotherm temperature [K]
///
pub fn attenuation_coef(freq: f64, hr: f64, ps: f64, ps0: f64, t: f64, t0: f64, t01: f64) -> f32 {
    let psat = ps0 * 10f64.powf(-6.8346 * (t01 / t).powf(1.261) + 4.6151);
    let h = ps0 * (hr / ps) * (psat / ps0);
    let f_ro = (24. + 4.04e4 * h * (0.02 + h) / (0.391 + h)) / ps0;
    let f_rn = (1. / ps0)
        * (9. + 280. * h * (-4.17 * ((t0 / t).powf(1. / 3.) - 1.)).exp())
        * (t0 / t).powf(1. / 2.);
    let f = freq / ps;

    let alpha = (f * f) / ps0
        * ps
        * (1.84 * (t / t0).powf(1. / 2.) * 1e-11
            + (t / t0).powf(-5. / 2.)
                * (0.01278 * (-2239.1 / t).exp() / (f_ro + f * f / f_ro)
                    + 0.1068 * (-3352. / t).exp() / (f_rn + f * f / f_rn)));
    (alpha * 1e-3) as f32
}
