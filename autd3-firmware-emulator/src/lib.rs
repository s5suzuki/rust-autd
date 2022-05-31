/*
 * File: lib.rs
 * Project: src
 * Created Date: 06/05/2022
 * Author: Shun Suzuki
 * -----
 * Last Modified: 31/05/2022
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2022 Shun Suzuki. All rights reserved.
 *
 */

use autd3_driver::{RxDatagram, TxDatagram};
use cpu::emulator::CPUEmulator;
use fpga::emulator::FPGAEmulator;

pub mod cpu;
pub mod fpga;

pub struct Emulator {
    devices: Vec<CPUEmulator>,
}

impl Emulator {
    pub fn new(n: usize) -> Self {
        let mut devices = Vec::with_capacity(n);
        (0..n).for_each(|i| devices.push(CPUEmulator::new(i)));
        Self { devices }
    }

    pub fn init(&mut self) {
        self.devices.iter_mut().for_each(|cpu| cpu.init());
    }

    pub fn send(&mut self, tx: &TxDatagram) {
        self.devices
            .iter_mut()
            .zip(tx.body().iter())
            .for_each(|(cpu, b)| cpu.send(tx.header(), b));
    }

    pub fn read(&mut self, rx: &mut RxDatagram) {
        self.devices
            .iter()
            .zip(rx.messages_mut().iter_mut())
            .for_each(|(cpu, r)| {
                r.msg_id = cpu.msg_id;
                r.ack = cpu.ack;
            });
    }

    pub fn cpu(&self, i: usize) -> &CPUEmulator {
        &self.devices[i]
    }

    pub fn cpus(&self) -> &[CPUEmulator] {
        &self.devices
    }

    pub fn fpga(&self, i: usize) -> &FPGAEmulator {
        &self.devices[i].fpga
    }
}
