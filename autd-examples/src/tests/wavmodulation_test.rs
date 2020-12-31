/*
 * File: wavmodulation_test.rs
 * Project: example
 * Created Date: 12/12/2019
 * Author: Shun Suzuki
 * -----
 * Last Modified: 31/12/2020
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2019 Hapis Lab. All rights reserved.
 *
 */

use std::error::Error;
use std::f64::consts::PI;
use std::ffi::OsString;

use autd::prelude::*;
use autd_wav_modulation::WavModulation;

pub fn wav_modulation_test<L: Link>(autd: &mut AUTD<L>) -> Result<(), Box<dyn Error>> {
    let mut g = FocalPointGain::create(Vector3::new(90., 70., 150.));
    autd.append_gain_sync(&mut g)?;

    let path = OsString::from("sine.wav");
    // write 150 Hz sine wave
    {
        let spec = hound::WavSpec {
            channels: 1,
            sample_rate: 4000,
            bits_per_sample: 8,
            sample_format: hound::SampleFormat::Int,
        };
        let mut writer = hound::WavWriter::create(&path, spec).unwrap();
        for t in (0..80).map(|x| x as f64 / 4000.0) {
            let sample = (t * 150. * 2.0 * PI).sin();
            let amplitude = std::i8::MAX as f64;
            let p = (sample * amplitude) as i8;
            writer.write_sample(p)?;
        }
    }

    let mut m = WavModulation::create(&path, 8, hound::SampleFormat::Int).unwrap();
    autd.append_modulation_sync(&mut m)?;

    Ok(())
}
