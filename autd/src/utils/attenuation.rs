/*
 * File: attenuation.rs
 * Project: common
 * Created Date: 14/05/2020
 * Author: Shun Suzuki
 * -----
 * Last Modified: 08/03/2021
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2020 Hapis Lab. All rights reserved.
 *
 */

use crate::Float;

const T0: Float = 293.15;
const T01: Float = 273.16;

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
///
pub fn attenuation_coef(freq: Float, hr: Float, ps: Float, ps0: Float, t: Float) -> Float {
    const TEN: Float = 10.;
    let psat = ps0 * TEN.powf(-6.8346 * (T01 / t).powf(1.261) + 4.6151);
    let h = ps0 * (hr / ps) * (psat / ps0);
    let f_ro = (24. + 4.04e4 * h * (0.02 + h) / (0.391 + h)) / ps0;
    let f_rn = (1. / ps0)
        * (9. + 280. * h * (-4.17 * ((T0 / t).powf(1. / 3.) - 1.)).exp())
        * (T0 / t).powf(1. / 2.);
    let f = freq / ps;

    let alpha = (f * f) / ps0
        * ps
        * (1.84 * (t / T0).powf(1. / 2.) * 1e-11
            + (t / T0).powf(-5. / 2.)
                * (0.01278 * (-2239.1 / t).exp() / (f_ro + f * f / f_ro)
                    + 0.1068 * (-3352. / t).exp() / (f_rn + f * f / f_rn)));
    alpha * 1e-3
}
