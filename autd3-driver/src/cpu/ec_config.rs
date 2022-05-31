/*
 * File: ec_config.rs
 * Project: src
 * Created Date: 27/04/2022
 * Author: Shun Suzuki
 * -----
 * Last Modified: 31/05/2022
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2022 Shun Suzuki. All rights reserved.
 *
 */

pub const HEADER_SIZE: usize = 128;
pub const BODY_SIZE: usize = 498;
pub const EC_OUTPUT_FRAME_SIZE: usize = HEADER_SIZE + BODY_SIZE;
pub const EC_INPUT_FRAME_SIZE: usize = 2;

pub const EC_SYNC0_CYCLE_TIME_MICRO_SEC: u32 = 500;
pub const EC_SYNC0_CYCLE_TIME_NANO_SEC: u32 = EC_SYNC0_CYCLE_TIME_MICRO_SEC * 1000;

pub const EC_DEVICE_PER_FRAME: usize = 2;
pub const EC_FRAME_LENGTH: usize =
    14 + 2 + (10 + EC_OUTPUT_FRAME_SIZE + EC_INPUT_FRAME_SIZE + 2) * EC_DEVICE_PER_FRAME + 4;
pub const EC_SPEED_BPS: f64 = 100.0 * 1000.0 * 1000.0;
pub const EC_TRAFFIC_DELAY: f64 = EC_FRAME_LENGTH as f64 * 8.0 / EC_SPEED_BPS;
