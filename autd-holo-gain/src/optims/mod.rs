/*
 * File: mod.rs
 * Project: multiple_foci
 * Created Date: 27/05/2020
 * Author: Shun Suzuki
 * -----
 * Last Modified: 31/12/2020
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2020 Hapis Lab. All rights reserved.
 *
 */

mod linear_synthesis;
pub mod macros;
mod matrix;
mod nls;

pub use linear_synthesis::*;
pub use matrix::*;
pub use nls::*;

use autd::{consts::DataArray, geometry::Geometry, prelude::Vector3, Float};

pub trait Optimizer: Send {
    fn optimize(
        &self,
        geometry: &Geometry,
        foci: &[Vector3],
        amps: &[Float],
        atten: Float,
        data: &mut [DataArray],
    );
}
