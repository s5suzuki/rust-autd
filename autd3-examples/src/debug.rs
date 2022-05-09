/*
 * File: debug.rs
 * Project: src
 * Created Date: 28/04/2022
 * Author: Shun Suzuki
 * -----
 * Last Modified: 08/05/2022
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2022 Hapis Lab. All rights reserved.
 *
 */

mod test_runner;
mod tests;

use std::fs::File;

use anyhow::Result;

use autd3::prelude::*;
use autd3_link_debug::Debug;
use simplelog::*;

fn main() -> Result<()> {
    CombinedLogger::init(vec![
        TermLogger::new(
            LevelFilter::Info,
            Config::default(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        ),
        WriteLogger::new(
            LevelFilter::Debug,
            Config::default(),
            File::create("autd3-example-debug.log").unwrap(),
        ),
    ])
    .unwrap();

    let mut geometry = GeometryBuilder::new().legacy_mode().build();
    geometry.add_device(Vector3::zeros(), Vector3::zeros());
    // geometry.add_device(Vector3::zeros(), Vector3::zeros());

    let link = Debug::new(geometry.num_devices());

    let autd = Controller::open(geometry, link).expect("Failed to open");

    run!(autd);

    Ok(())
}
