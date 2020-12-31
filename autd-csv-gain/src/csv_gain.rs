/*
 * File: csv_gain.rs
 * Project: gain
 * Created Date: 02/12/2019
 * Author: Shun Suzuki
 * -----
 * Last Modified: 31/12/2020
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2019 Hapis Lab. All rights reserved.
 *
 */

use std::error::Error;
use std::ffi::OsString;
use std::fmt;
use std::fs::File;

use autd::gain::*;
use autd::geometry::Geometry;
use autd::{consts::*, Float};

#[derive(Debug)]
pub enum CsvGainError {
    ParseError,
}

impl fmt::Display for CsvGainError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::ParseError => write!(f, "The file must consist of two columns, normalized amp and phase, with delimiter ','."),
        }
    }
}

impl Error for CsvGainError {
    fn description(&self) -> &str {
        match *self {
            Self::ParseError => "The file must consist of two columns, normalized amp and phase, with delimiter ','.",
        }
    }

    fn cause(&self) -> Option<&dyn Error> {
        match *self {
            Self::ParseError => None,
        }
    }
}

pub struct CsvGain {
    data: Option<Vec<DataArray>>,
}

impl CsvGain {
    pub fn create(file_path: &OsString) -> Result<Self, Box<dyn Error>> {
        let mut data = Vec::new();
        let file = File::open(file_path)?;
        let mut rdr = csv::Reader::from_reader(file);

        let mut buf: DataArray = unsafe { std::mem::zeroed() };
        let mut idx = 0;
        for result in rdr.records() {
            let record = result?;
            if record.len() != 2 {
                return Err(CsvGainError::ParseError.into());
            }
            let amp: Float = record[0].parse()?;
            let phase: Float = record[1].parse()?;
            let phase = (phase * 255.0) as u16;
            let d = (adjust_amp(amp) as u16) << 8;
            buf[idx] = d | phase;
            idx += 1;
            if idx == NUM_TRANS_IN_UNIT {
                data.push(buf);
                buf = unsafe { std::mem::zeroed() };
                idx = 0;
            }
        }
        Ok(Self { data: Some(data) })
    }
}

impl Gain for CsvGain {
    fn get_data(&self) -> &[DataArray] {
        assert!(self.data.is_some());
        match &self.data {
            Some(data) => data,
            None => panic!(),
        }
    }

    fn build(&mut self, geometry: &Geometry) {
        let ndevice = geometry.num_devices();
        if let Some(data) = &mut self.data {
            let buf: DataArray = unsafe { std::mem::zeroed() };
            data.resize(ndevice, buf);
        }
    }
}
