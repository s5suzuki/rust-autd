/*
 * File: hologain_test.rs
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

use colored::*;

use autd::prelude::*;
use autd_holo_gain::*;

use std::error::Error;
use std::io;

macro_rules! sel_opt {
    ([$(($idx:tt, $opt:ty)),*]) => {{
        let opts = vec![$(($idx, stringify!($opt)),)*];
        for (index, opt) in &opts {
            println!("[{}]: {}", index, opt);
        }
        let mut s = String::new();
        io::stdin().read_line(&mut s)?;
        match s.trim().parse() {
            Ok(num) if num < opts.len() => num,
            _ => 0,
        }
    }}
}

macro_rules! choose_opt {
    ($sel:ident, $autd:ident, [$(($idx:tt, $opt:ty)),*]) => {
        match $sel {
            $($idx => {
                print!(
                    "{}{}\n",
                    "Optimization method is "
                        .green(),
                    stringify!($opt)
                        .green()
                        .bold(),
                );
                let opt = <$opt>::default();
                let mut g = HoloGain::create(
                    vec![Vector3::new(70., 70., 150.), Vector3::new(110., 70., 150.)],
                    vec![1., 1.],
                    opt,
                );
                $autd.append_gain_sync(&mut g)?;
            },)*
            _ => {
                let opt = Horn::default();
                let mut g = HoloGain::create(
                    vec![Vector3::new(70., 70., 150.), Vector3::new(110., 70., 150.)],
                    vec![1., 1.],
                    opt,
                );
                $autd.append_gain_sync(&mut g)?;
            }
        }
    }
}

pub fn hologain_test<L: Link>(autd: &mut AUTD<L>) -> Result<(), Box<dyn Error>> {
    print!("{}", "Choose optimizing method (default is Horn)\n".green());
    let sel = sel_opt!([
        (0, Horn),
        (1, Long),
        (2, Naive),
        (3, GS),
        (4, GSPAT),
        (5, LM),
        (6, APO)
    ]);
    choose_opt!(
        sel,
        autd,
        [
            (0, Horn),
            (1, Long),
            (2, Naive),
            (3, GS),
            (4, GSPAT),
            (5, LM),
            (6, APO)
        ]
    );

    let mut m = SineModulation::create(150);
    autd.append_modulation_sync(&mut m)?;
    Ok(())
}
