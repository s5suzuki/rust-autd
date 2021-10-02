/*
 * File: stm_controller.rs
 * Project: src
 * Created Date: 26/05/2021
 * Author: Shun Suzuki
 * -----
 * Last Modified: 02/10/2021
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2021 Hapis Lab. All rights reserved.
 *
 */

use crate::controller::{Controller, ControllerProps};
use anyhow::Result;
use autd3_core::{
    ec_config::EC_OUTPUT_FRAME_SIZE,
    gain::Gain,
    hardware_defined::{CPUControlFlags, CommandType, FPGAControlFlags},
    link::Link,
    logic::Logic,
};
use autd3_timer::{Timer, TimerCallback};
use std::sync::atomic::{AtomicBool, Ordering};

pub(crate) struct StmTimerCallback<L: Link> {
    pub(crate) link: L,
    pub(crate) buffers: Vec<Vec<u8>>,
    idx: usize,
    lock: AtomicBool,
}

impl<L: Link> StmTimerCallback<L> {
    pub fn new(link: L) -> Self {
        Self {
            link,
            buffers: vec![],
            idx: 0,
            lock: AtomicBool::new(false),
        }
    }

    pub fn add(&mut self, data: Vec<u8>) {
        self.buffers.push(data);
    }

    pub fn clear(&mut self) {
        self.buffers.clear();
        self.idx = 0;
    }
}

impl<L: Link> TimerCallback for StmTimerCallback<L> {
    fn rt_thread(&mut self) {
        if let Ok(false) =
            self.lock
                .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        {
            let data = &self.buffers[self.idx];
            self.link.send(data).unwrap();
            self.idx = (self.idx + 1) % self.buffers.len();
            self.lock.store(false, Ordering::Release);
        }
    }
}

/// Controller to Spatio-temporal modulation (STM)
pub struct StmController<L: Link> {
    pub(crate) callback: StmTimerCallback<L>,
    pub(crate) props: ControllerProps,
}

impl<L: Link> StmController<L> {
    fn new(callback: StmTimerCallback<L>, props: ControllerProps) -> Self {
        Self { callback, props }
    }

    /// Return normal controller
    pub fn controller(self) -> Controller<L> {
        Controller::new(self.callback.link, self.props)
    }

    /// Add gain for STM
    ///
    /// # Arguments
    ///
    /// * `g` - Gain
    ///
    pub fn add<G: Gain>(&mut self, g: &mut G) -> Result<()> {
        g.build(&self.props.geometry)?;

        let mut build_buf = vec![0x00; self.props.geometry.num_devices() * EC_OUTPUT_FRAME_SIZE];

        let mut _size = 0;
        Logic::pack_body(g, &mut build_buf, &mut _size);

        let mut _msg_id = 0;
        Logic::pack_header(
            CommandType::Op,
            self.ctrl_flag(),
            CPUControlFlags::NONE,
            &mut build_buf,
            &mut _msg_id,
        );

        self.callback.add(build_buf);

        Ok(())
    }

    /// Start Spatio-Temporal Modulation
    ///
    /// Start STM by switching gains appended by [add](#method.add) at the `freq`. The accuracy depends on the computer, for example, about 1ms on Windows. Note that it is affected by interruptions, and so on.
    ///
    /// # Arguments
    ///
    /// * `freq` - freq Frequency of STM modulation
    ///
    pub fn start(self, freq: f64) -> Result<StmTimer<L>> {
        let len = self.callback.buffers.len();
        assert!(len != 0);
        let itvl_ns = 1000000000. / freq / len as f64;
        let timer = StmTimer {
            timer: Timer::start(self.callback, itvl_ns as u32)?,
            props: self.props,
        };
        Ok(timer)
    }

    /// Finish Spatio-Temporal Modulation
    ///
    /// Added gains will be removed.
    pub fn finish(&mut self) {
        self.callback.clear();
    }

    fn ctrl_flag(&self) -> FPGAControlFlags {
        let mut header = FPGAControlFlags::OUTPUT_ENABLE;
        if self.props.output_balance {
            header |= FPGAControlFlags::OUTPUT_BALANCE;
        }
        if self.props.silent_mode {
            header |= FPGAControlFlags::SILENT;
        }
        if self.props.force_fan {
            header |= FPGAControlFlags::FORCE_FAN;
        }
        header
    }
}

/// STM timer handler
pub struct StmTimer<L: Link> {
    pub(crate) timer: Box<Timer<StmTimerCallback<L>>>,
    pub(crate) props: ControllerProps,
}

impl<L: Link> StmTimer<L> {
    /// Stop STM
    pub fn stop(self) -> Result<StmController<L>> {
        let cb = self.timer.close()?;
        Ok(StmController::new(cb, self.props))
    }
}
