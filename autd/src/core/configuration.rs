/*
 * File: configuration.rs
 * Project: core
 * Created Date: 30/12/2020
 * Author: Shun Suzuki
 * -----
 * Last Modified: 30/12/2020
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2020 Hapis Lab. All rights reserved.
 *
 */

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ModSamplingFreq {
    Smpl125Hz = 125,
    Smpl250Hz = 250,
    Smpl500Hz = 500,
    Smpl1Khz = 1000,
    Smpl2Khz = 2000,
    Smpl4Khz = 4000,
    Smpl8Khz = 8000,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ModBufSize {
    Buf125 = 125,
    Buf250 = 250,
    Buf500 = 500,
    Buf1000 = 1000,
    Buf2000 = 2000,
    Buf4000 = 4000,
    Buf8000 = 8000,
    Buf16000 = 16000,
    Buf32000 = 32000,
}

#[derive(Clone, Copy)]
pub struct Configuration {
    smpl_freq: ModSamplingFreq,
    buf_size: ModBufSize,
}

impl Configuration {
    pub fn new(smpl_freq: ModSamplingFreq, buf_size: ModBufSize) -> Self {
        Self {
            smpl_freq,
            buf_size,
        }
    }

    pub fn sampling_frequency(&self) -> ModSamplingFreq {
        self.smpl_freq
    }

    pub fn buf_size(&self) -> ModBufSize {
        self.buf_size
    }
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            smpl_freq: ModSamplingFreq::Smpl4Khz,
            buf_size: ModBufSize::Buf4000,
        }
    }
}
