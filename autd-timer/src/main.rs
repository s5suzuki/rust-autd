/*
 * File: main.rs
 * Project: src
 * Created Date: 23/05/2020
 * Author: Shun Suzuki
 * -----
 * Last Modified: 24/05/2020
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2020 Hapis Lab. All rights reserved.
 *
 */

use autd_timer::Timer;

fn main() {
    use std::{thread, time};

    let st = std::time::Instant::now();
    let mut timer = Timer::new();

    timer.start(
        move || println!("{}", st.elapsed().as_micros() as f64 / 1000.0),
        1000 * 1000,
    );

    let ten_millis = time::Duration::from_millis(100);

    thread::sleep(ten_millis);

    timer.close();

    println!("fin");
}
