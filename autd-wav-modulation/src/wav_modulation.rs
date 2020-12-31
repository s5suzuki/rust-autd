/*
 * File: wav_modulation.rs
 * Project: modulation
 * Created Date: 03/12/2019
 * Author: Shun Suzuki
 * -----
 * Last Modified: 31/12/2020
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2019 Hapis Lab. All rights reserved.
 *
 */

use autd::modulation::Modulation;
use autd::{consts::MOD_BUF_SIZE, Configuration};

use hound::SampleFormat;
use std::ffi::OsString;
use std::{error::Error, vec};

pub struct WavModulation {
    raw_buf: Vec<u8>,
    buffer: Vec<u8>,
    sample_rate: u32,
    sent: usize,
}

impl WavModulation {
    pub fn create(
        path: &OsString,
        bits_per_sample: u16,
        sample_format: SampleFormat,
    ) -> Result<Self, Box<dyn Error>> {
        let mut reader = hound::WavReader::open(path)?;
        let sample_rate = reader.spec().sample_rate;
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

        Ok(Self {
            raw_buf: buffer,
            buffer: vec![],
            sample_rate,
            sent: 0,
        })
    }
}

impl Modulation for WavModulation {
    fn build(&mut self, config: Configuration) {
        let mod_sf = config.sampling_frequency() as usize;
        let mod_buf_size = config.buf_size() as usize;

        // down sampling
        let freq_ratio = mod_sf as f64 / self.sample_rate as f64;
        let mut buffer_size = (self.raw_buf.len() as f64 * freq_ratio) as usize;
        if buffer_size > mod_buf_size {
            buffer_size = mod_buf_size;
        }

        let mut sample_buf = Vec::with_capacity(buffer_size);
        for i in 0..sample_buf.len() {
            let idx = (i as f64 / freq_ratio) as usize;
            sample_buf.push(self.raw_buf[idx]);
        }

        self.buffer = sample_buf;
    }

    fn buffer(&self) -> &[u8] {
        &self.buffer
    }

    fn sent(&mut self) -> &mut usize {
        &mut self.sent
    }
}
