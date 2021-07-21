/*
 * File: build.rs
 * Project: autd3-soem-link
 * Created Date: 26/05/2021
 * Author: Shun Suzuki
 * -----
 * Last Modified: 21/07/2021
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2021 Hapis Lab. All rights reserved.
 *
 */

use std::env;
use std::path::PathBuf;

macro_rules! add {
    ($path:expr, $p:ident, $work: expr) => {
        for entry in glob::glob($path).unwrap() {
            match entry {
                Ok($p) => {
                    $work;
                }
                Err(e) => println!("{:?}", e),
            }
        }
    };
}

#[cfg(target_os = "windows")]
fn main() {
    let home_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    println!("cargo:rustc-link-lib=winmm");
    println!("cargo:rustc-link-lib=ws2_32");
    println!(
        "cargo:rustc-link-search={}\\deps\\SOEM\\oshw\\win32\\wpcap\\Lib\\x64",
        home_dir
    );
    println!("cargo:rustc-link-lib=Packet");
    println!("cargo:rustc-link-lib=wpcap");

    let mut build = cc::Build::new();
    build.flag("/DWIN32").warnings(true).cpp(false);
    add!("deps/SOEM/soem/*.c", path, build.file(path));
    add!("deps/SOEM/osal/win32/*.c", path, build.file(path));
    add!("deps/SOEM/oshw/win32/*.c", path, build.file(path));
    build
        .include("deps/SOEM/soem")
        .include("deps/SOEM/osal")
        .include("deps/SOEM/osal/win32")
        .include("deps/SOEM/oshw/win32")
        .include("deps/SOEM/oshw/win32/wpcap/Include")
        .include("deps/SOEM/oshw/win32/wpcap/Include/pcap")
        .compile("soem");

    let bindings = bindgen::Builder::default()
        .clang_arg("-DWIN32")
        .clang_arg("-Ideps/SOEM/soem")
        .clang_arg("-Ideps/SOEM/osal")
        .clang_arg("-Ideps/SOEM/osal/win32")
        .clang_arg("-Ideps/SOEM/oshw/win32")
        .clang_arg("-Ideps/SOEM/oshw/win32/wpcap/Include")
        .clang_arg("-Ideps/SOEM/oshw/win32/wpcap/Include/pcap")
        .header("deps/SOEM/osal/win32/osal_defs.h")
        .header("deps/SOEM/soem/ethercattype.h")
        .header("deps/SOEM/oshw/win32/nicdrv.h")
        .header("deps/SOEM/soem/ethercatmain.h")
        .header("deps/SOEM/soem/ethercatdc.h")
        .header("deps/SOEM/soem/ethercatconfig.h")
        .allowlist_function("ec_init")
        .allowlist_function("ec_find_adapters")
        .allowlist_function("ec_send_processdata")
        .allowlist_function("ec_receive_processdata")
        .allowlist_function("ec_config")
        .allowlist_function("ec_dcsync0")
        .allowlist_function("ec_configdc")
        .allowlist_function("ec_writestate")
        .allowlist_function("ec_statecheck")
        .allowlist_function("ec_close")
        .allowlist_function("ec_readstate")
        .allowlist_function("ec_reconfig_slave")
        .allowlist_function("ec_recover_slave")
        .allowlist_var("ec_slave")
        .allowlist_var("ec_group")
        .allowlist_var("ec_slavecount")
        .allowlist_var("EC_TIMEOUTSTATE")
        .allowlist_var("EC_TIMEOUTRET")
        .allowlist_type("ec_state")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("soem_bindings.rs"))
        .expect("Couldn't write bindings!");
}

#[cfg(target_os = "macos")]
fn main() {
    println!("cargo:rustc-link-lib=pthread");
    println!("cargo:rustc-link-lib=pcap");

    let mut build = cc::Build::new();
    build.warnings(true).cpp(false);
    add!("deps/SOEM/soem/*.c", path, build.file(path));
    add!("deps/SOEM/osal/macosx/*.c", path, build.file(path));
    add!("deps/SOEM/oshw/macosx/*.c", path, build.file(path));
    build
        .include("deps/SOEM/soem")
        .include("deps/SOEM/osal")
        .include("deps/SOEM/osal/macosx")
        .include("deps/SOEM/oshw/macosx")
        .compile("soem");

    let bindings = bindgen::Builder::default()
        .clang_arg("-Ideps/SOEM/soem")
        .clang_arg("-Ideps/SOEM/osal")
        .clang_arg("-Ideps/SOEM/osal/macosx")
        .clang_arg("-Ideps/SOEM/oshw/macosx")
        .header("deps/SOEM/osal/macosx/osal_defs.h")
        .header("deps/SOEM/soem/ethercattype.h")
        .header("deps/SOEM/oshw/macosx/nicdrv.h")
        .header("deps/SOEM/soem/ethercatmain.h")
        .header("deps/SOEM/soem/ethercatdc.h")
        .header("deps/SOEM/soem/ethercatconfig.h")
        .allowlist_function("ec_init")
        .allowlist_function("ec_find_adapters")
        .allowlist_function("ec_send_processdata")
        .allowlist_function("ec_receive_processdata")
        .allowlist_function("ec_config")
        .allowlist_function("ec_dcsync0")
        .allowlist_function("ec_configdc")
        .allowlist_function("ec_writestate")
        .allowlist_function("ec_statecheck")
        .allowlist_function("ec_close")
        .allowlist_function("ec_readstate")
        .allowlist_function("ec_reconfig_slave")
        .allowlist_function("ec_recover_slave")
        .allowlist_var("ec_slave")
        .allowlist_var("ec_group")
        .allowlist_var("ec_slavecount")
        .allowlist_var("EC_TIMEOUTSTATE")
        .allowlist_var("EC_TIMEOUTRET")
        .allowlist_type("ec_state")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("soem_bindings.rs"))
        .expect("Couldn't write bindings!");
}

#[cfg(target_os = "linux")]
fn main() {
    println!("cargo:rustc-link-lib=pthread");
    println!("cargo:rustc-link-lib=rt");

    let mut build = cc::Build::new();
    build.warnings(true).cpp(false);
    add!("deps/SOEM/soem/*.c", path, build.file(path));
    add!("deps/SOEM/osal/linux/*.c", path, build.file(path));
    add!("deps/SOEM/oshw/linux/*.c", path, build.file(path));
    build
        .include("deps/SOEM/soem")
        .include("deps/SOEM/osal")
        .include("deps/SOEM/osal/linux")
        .include("deps/SOEM/oshw/linux")
        .compile("soem");

    let bindings = bindgen::Builder::default()
        .clang_arg("-Ideps/SOEM/soem")
        .clang_arg("-Ideps/SOEM/osal")
        .clang_arg("-Ideps/SOEM/osal/linux")
        .clang_arg("-Ideps/SOEM/oshw/linux")
        .header("deps/SOEM/osal/linux/osal_defs.h")
        .header("deps/SOEM/soem/ethercattype.h")
        .header("deps/SOEM/oshw/linux/nicdrv.h")
        .header("deps/SOEM/soem/ethercatmain.h")
        .header("deps/SOEM/soem/ethercatdc.h")
        .header("deps/SOEM/soem/ethercatconfig.h")
        .allowlist_function("ec_init")
        .allowlist_function("ec_find_adapters")
        .allowlist_function("ec_send_processdata")
        .allowlist_function("ec_receive_processdata")
        .allowlist_function("ec_config")
        .allowlist_function("ec_dcsync0")
        .allowlist_function("ec_configdc")
        .allowlist_function("ec_writestate")
        .allowlist_function("ec_statecheck")
        .allowlist_function("ec_close")
        .allowlist_function("ec_readstate")
        .allowlist_function("ec_reconfig_slave")
        .allowlist_function("ec_recover_slave")
        .allowlist_var("ec_slave")
        .allowlist_var("ec_group")
        .allowlist_var("ec_slavecount")
        .allowlist_var("EC_TIMEOUTSTATE")
        .allowlist_var("EC_TIMEOUTRET")
        .allowlist_type("ec_state")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("soem_bindings.rs"))
        .expect("Couldn't write bindings!");
}
