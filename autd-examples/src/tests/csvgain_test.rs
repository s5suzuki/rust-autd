/*
 * File: csvgain_test.rs
 * Project: example
 * Created Date: 12/12/2019
 * Author: Shun Suzuki
 * -----
 * Last Modified: 07/08/2020
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2019 Hapis Lab. All rights reserved.
 *
 */

use std::ffi::OsString;

use std::error::Error;

use autd::prelude::*;
use autd_csv_gain::CsvGain;

pub fn csvgain_test(autd: &mut AUTD) -> Result<(), Box<dyn Error>> {
    let path = OsString::from("csv_gain_focal.csv");
    //write
    {
        let mut wtr = csv::Writer::from_path(&path).unwrap();
        let x = 90.;
        let y = 70.;
        let z = 150.;
        for ty_idx in 0..NUM_TRANS_Y {
            for tx_idx in 0..NUM_TRANS_X {
                if !autd::geometry::is_missing_transducer(tx_idx, ty_idx) {
                    let tx = tx_idx as f64 * TRANS_SIZE;
                    let ty = ty_idx as f64 * TRANS_SIZE;
                    let dist = ((tx - x) * (tx - x) + (ty - y) * (ty - y) + z * z).sqrt();
                    let phase = 1.0 - (dist % ULTRASOUND_WAVELENGTH) / ULTRASOUND_WAVELENGTH;
                    let amp = 1.0;
                    wtr.serialize([amp, phase])?; // The file must consist of two columns, normalized amp and phase, with delimiter ','.
                }
            }
        }
        wtr.flush()?;
    }
    let g = CsvGain::create(&path)?;
    autd.append_gain_sync(g);

    let m = SineModulation::create(150);
    autd.append_modulation_sync(m);
    Ok(())
}
