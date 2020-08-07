/*
 * File: twincat.rs
 * Project: examples
 * Created Date: 25/05/2020
 * Author: Shun Suzuki
 * -----
 * Last Modified: 07/08/2020
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2020 Hapis Lab. All rights reserved.
 *
 */

mod tests;

use autd::prelude::*;
#[cfg(target_os = "windows")]
use autd_twincat_link::LocalTwinCATLink;
#[cfg(not(target_os = "windows"))]
use autd_twincat_link::RemoteTwinCATLink;
use tests::*;

#[cfg(target_os = "windows")]
fn main() {
    let mut autd = AUTD::create();

    autd.geometry()
        .add_device(Vector3::zeros(), Vector3::zeros());

    let link = LocalTwinCATLink::new();

    autd.open(link).expect("Failed to open");

    run(autd);
}

#[cfg(not(target_os = "windows"))]
fn main() {
    use std::io;
    use std::io::Write;

    let mut autd = AUTD::create();

    autd.geometry()
        .add_device(Vector3::zeros(), Vector3::zeros());

    print!("Enter a remote TwinCAT AUTD Server address: ");
    io::stdout().flush().unwrap();

    let mut addr = String::new();
    io::stdin().read_line(&mut addr).unwrap();

    let link = RemoteTwinCATLink::new(&addr);
    autd.open(link).expect("Failed to open");

    run(autd);
}
