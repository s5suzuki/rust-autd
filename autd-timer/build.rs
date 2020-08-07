/*
 * File: build.rs
 * Project: autd-timer
 * Created Date: 23/05/2020
 * Author: Shun Suzuki
 * -----
 * Last Modified: 23/05/2020
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2020 Hapis Lab. All rights reserved.
 *
 */

#[cfg(target_os = "windows")]
fn main() {
    println!("cargo:rustc-link-lib=winmm");
}

#[cfg(target_os = "linux")]
fn main() {
    println!("cargo:rustc-link-lib=rt");
}

#[cfg(target_os = "macos")]
fn main() {
    println!("cargo:rustc-link-lib=pthread");
}
