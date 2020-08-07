/*
 * File: csv_gain.rs
 * Project: gain
 * Created Date: 02/12/2019
 * Author: Shun Suzuki
 * -----
 * Last Modified: 07/08/2020
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2019 Hapis Lab. All rights reserved.
 *
 */

use std::error::Error;
use std::ffi::OsString;
use std::fmt;
use std::fs::File;

use autd::consts::*;
use autd::gain::*;
use autd::geometry::Geometry;

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
    data: Option<Vec<u8>>,
}

impl CsvGain {
    pub fn create(file_path: &OsString) -> Result<Box<Self>, Box<dyn Error>> {
        let mut data = Vec::new();
        let file = File::open(file_path)?;
        let mut rdr = csv::Reader::from_reader(file);
        for result in rdr.records() {
            let record = result?;
            if record.len() != 2 {
                return Err(CsvGainError::ParseError.into());
            }
            let amp: f64 = record[0].parse()?;
            let phase: f64 = record[1].parse()?;
            let amp = (amp * 255.0) as u8;
            let phase = (phase * 255.0) as u8;
            let d = adjust_amp(amp);
            let s = phase;
            data.push(s);
            data.push(d);
        }
        Ok(Box::new(Self { data: Some(data) }))
    }
}

impl Gain for CsvGain {
    fn get_data(&self) -> &[u8] {
        assert!(self.data.is_some());
        match &self.data {
            Some(data) => data,
            None => panic!(),
        }
    }

    fn build(&mut self, geometry: &Geometry) {
        let ndevice = geometry.num_devices();
        let ntrans = NUM_TRANS_IN_UNIT * ndevice;

        if let Some(data) = &mut self.data {
            data.resize(ntrans * 2, 0);
        }
    }
}
