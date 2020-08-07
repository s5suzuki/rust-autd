/*
 * File: wav_modulation.rs
 * Project: modulation
 * Created Date: 03/12/2019
 * Author: Shun Suzuki
 * -----
 * Last Modified: 07/08/2020
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2019 Hapis Lab. All rights reserved.
 *
 */

use autd::consts::MOD_BUF_SIZE;
use autd::modulation::Modulation;

use hound::SampleFormat;
use std::error::Error;
use std::ffi::OsString;

pub struct WavModulation {}

impl WavModulation {
    pub fn create(
        path: &OsString,
        bits_per_sample: u16,
        sample_format: SampleFormat,
    ) -> Result<Modulation, Box<dyn Error>> {
        let mut reader = hound::WavReader::open(path)?;
        let sample_iter: Vec<u8> = match (sample_format, bits_per_sample) {
            (SampleFormat::Int, 8) => reader
                .samples::<i32>()
                .map(|i| i.unwrap() as i32)
                .map(|i| (i - std::i8::MIN as i32) as u8)
                .collect(),
            (SampleFormat::Int, 16) => reader
                .samples::<i32>()
                .map(|i| (i.unwrap() as i32) / (i32::pow(2, 8)))
                .map(|i| (i - std::i8::MIN as i32) as u8)
                .collect(),
            (SampleFormat::Int, 24) => reader
                .samples::<i32>()
                .map(|i| (i.unwrap() as i32) / (i32::pow(2, 16)))
                .map(|i| (i - std::i8::MIN as i32) as u8)
                .collect(),
            (SampleFormat::Int, 32) => reader
                .samples::<i32>()
                .map(|i| i.unwrap() / i32::pow(2, 24))
                .map(|i| (i - std::i8::MIN as i32) as u8)
                .collect(),
            (SampleFormat::Float, 32) => reader
                .samples::<f32>()
                .map(|i| i.unwrap())
                .map(|i| ((i + 1.0) / 2.0 * 255.0) as u8)
                .collect(),
            _ => return Err(From::from("Not supported format.")),
        };
        let size = std::cmp::min(MOD_BUF_SIZE as usize, sample_iter.len());
        let mut buffer = Vec::with_capacity(size);
        buffer.extend_from_slice(&sample_iter[0..size]);

        Ok(Modulation::new(buffer))
    }
}
