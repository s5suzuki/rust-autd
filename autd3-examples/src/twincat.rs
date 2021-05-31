/*
 * File: twincat.rs
 * Project: src
 * Created Date: 31/05/2021
 * Author: Shun Suzuki
 * -----
 * Last Modified: 31/05/2021
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2021 Hapis Lab. All rights reserved.
 * 
 */


mod test_runner;
mod tests;

use autd3::prelude::*;
use autd3_twincat_link::TwinCatLink;
use test_runner::run;

async fn main_task() {
    let mut geometry = Geometry::new();
    geometry.add_device(Vector3::zeros(), Vector3::zeros());

    let link = TwinCatLink::new();
    let autd = Controller::open(geometry, link).expect("Failed to open");

    run(autd).await.expect("Some error occurred.");
}

fn main() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async { main_task().await });
}
