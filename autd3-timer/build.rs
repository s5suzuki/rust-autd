/*
 * File: build.rs
 * Project: autd3-timer
 * Created Date: 24/05/2021
 * Author: Shun Suzuki
 * -----
 * Last Modified: 02/10/2021
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2021 Hapis Lab. All rights reserved.
 *
 */

#[cfg(target_os = "windows")]
fn main() {
    windows::build!(
        Windows::Win32::Media::Multimedia::timeBeginPeriod,
        Windows::Win32::Media::Multimedia::timeEndPeriod,
        Windows::Win32::System::SystemServices::timeSetEvent,
        Windows::Win32::System::SystemServices::timeKillEvent,
        Windows::Win32::System::SystemServices::LPTIMECALLBACK,
        Windows::Win32::System::SystemServices::TIME_PERIODIC,
        Windows::Win32::System::SystemServices::TIME_CALLBACK_FUNCTION,
        Windows::Win32::System::SystemServices::TIME_KILL_SYNCHRONOUS,
        Windows::Win32::System::Threading::*,
        Windows::Win32::Media::Multimedia::TIMERR_NOERROR
    );
}

#[cfg(target_os = "linux")]
fn main() {
    println!("cargo:rustc-link-lib=rt");
}

#[cfg(target_os = "macos")]
fn main() {
    println!("cargo:rustc-link-lib=pthread");
}
