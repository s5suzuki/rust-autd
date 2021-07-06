/*
 * File: native_methods.rs
 * Project: src
 * Created Date: 30/08/2019
 * Author: Shun Suzuki
 * -----
 * Last Modified: 06/07/2021
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2019 Hapis Lab. All rights reserved.
 *
 */

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]
#![allow(clippy::upper_case_acronyms)]
#![allow(unaligned_references)]
#![allow(deref_nullptr)]
include!(concat!(env!("OUT_DIR"), "/soem_bindings.rs"));
