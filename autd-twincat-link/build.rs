/*
 * File: build.rs
 * Project: src
 * Created Date: 16/12/2019
 * Author: Shun Suzuki
 * -----
 * Last Modified: 24/05/2020
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2019 Hapis Lab. All rights reserved.
 *
 */

#[cfg(target_os = "windows")]
fn main() {
    println!("cargo:rustc-link-lib=ws2_32");
    cc::Build::new()
        .warnings(true)
        .cpp(true)
        .static_flag(true)
        .file("deps/ADS/AdsLib/AdsDef.cpp")
        .file("deps/ADS/AdsLib/AdsLib.cpp")
        .file("deps/ADS/AdsLib/AmsConnection.cpp")
        .file("deps/ADS/AdsLib/AmsPort.cpp")
        .file("deps/ADS/AdsLib/AmsRouter.cpp")
        .file("deps/ADS/AdsLib/Frame.cpp")
        .file("deps/ADS/AdsLib/Log.cpp")
        .file("deps/ADS/AdsLib/NotificationDispatcher.cpp")
        .file("deps/ADS/AdsLib/Sockets.cpp")
        .include("deps/ADS/AdsLib")
        .compile("libads.a");
}

#[cfg(target_os = "linux")]
fn main() {
    cc::Build::new()
        .warnings(true)
        .cpp(true)
        .static_flag(true)
        .file("deps/ADS/AdsLib/AdsDef.cpp")
        .file("deps/ADS/AdsLib/AdsLib.cpp")
        .file("deps/ADS/AdsLib/AmsConnection.cpp")
        .file("deps/ADS/AdsLib/AmsPort.cpp")
        .file("deps/ADS/AdsLib/AmsRouter.cpp")
        .file("deps/ADS/AdsLib/Frame.cpp")
        .file("deps/ADS/AdsLib/Log.cpp")
        .file("deps/ADS/AdsLib/NotificationDispatcher.cpp")
        .file("deps/ADS/AdsLib/Sockets.cpp")
        .include("deps/ADS/AdsLib")
        .compile("libads.a");
}

#[cfg(target_os = "macos")]
fn main() {
    cc::Build::new()
        .warnings(true)
        .cpp(true)
        .static_flag(true)
        .flag("-std=c++17")
        .file("deps/ADS/AdsLib/AdsDef.cpp")
        .file("deps/ADS/AdsLib/AdsLib.cpp")
        .file("deps/ADS/AdsLib/AmsConnection.cpp")
        .file("deps/ADS/AdsLib/AmsPort.cpp")
        .file("deps/ADS/AdsLib/AmsRouter.cpp")
        .file("deps/ADS/AdsLib/Frame.cpp")
        .file("deps/ADS/AdsLib/Log.cpp")
        .file("deps/ADS/AdsLib/NotificationDispatcher.cpp")
        .file("deps/ADS/AdsLib/Sockets.cpp")
        .include("deps/ADS/AdsLib")
        .compile("libads.a");
}
