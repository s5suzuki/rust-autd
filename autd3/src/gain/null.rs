/*
 * File: null.rs
 * Project: gain
 * Created Date: 26/05/2021
 * Author: Shun Suzuki
 * -----
 * Last Modified: 24/11/2021
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2021 Hapis Lab. All rights reserved.
 *
 */

use anyhow::Result;
use autd3_core::{
    gain::{Gain, GainData},
    geometry::Geometry,
};
use autd3_traits::Gain;
use std::vec;

/// Gain with no output
#[derive(Gain)]
pub struct Null {
    data: Vec<GainData>,
    built: bool,
}

impl Null {
    /// constructor
    pub fn new() -> Self {
        Self {
            data: vec![],
            built: false,
        }
    }

    #[allow(clippy::unnecessary_wraps)]
    fn calc(&mut self, _geometry: &Geometry) -> Result<()> {
        Ok(())
    }
}

impl Default for Null {
    fn default() -> Self {
        Self::new()
    }
}
