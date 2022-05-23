/*
 * File: debug_link.rs
 * Project: src
 * Created Date: 28/04/2022
 * Author: Shun Suzuki
 * -----
 * Last Modified: 23/05/2022
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2022 Hapis Lab. All rights reserved.
 *
 */

use autd3_core::{link::Link, CPUControlFlags, RxDatagram, TxDatagram};
use autd3_firmware_emulator::Emulator;

pub struct Debug {
    emulator: Emulator,
}

impl Debug {
    pub fn new(n: usize) -> Self {
        Self {
            emulator: Emulator::new(n),
        }
    }
}

impl Link for Debug {
    fn open(&mut self) -> anyhow::Result<()> {
        log::info!("Open Debug link");

        self.emulator.init();
        log::info!("Initialize emulator");

        Ok(())
    }

    fn close(&mut self) -> anyhow::Result<()> {
        log::info!("Close Debug link");
        Ok(())
    }

    fn send(&mut self, tx: &TxDatagram) -> anyhow::Result<bool> {
        log::info!("Send data");
        log::info!("\tCPU Flag: {:?}", tx.header().cpu_flag);
        log::info!("\tFPGA Flag: {:?}", tx.header().fpga_flag);

        self.emulator.send(tx);

        self.emulator.cpus().iter().for_each(|cpu| {
            log::info!("Status: {}", cpu.id());
            let fpga = cpu.fpga();
            if fpga.is_stm_mode() {
                if fpga.is_stm_gain_mode() {
                    if fpga.is_legacy_mode() {
                        log::info!("\tGain STM Legacy mode");
                    } else {
                        log::info!("\tGain STM mode");
                    }
                } else {
                    log::info!("\tPoint STM mode");
                }
                if tx.header().cpu_flag.contains(CPUControlFlags::STM_BEGIN) {
                    log::info!("\t\tSTM BEGIN");
                }
                if tx.header().cpu_flag.contains(CPUControlFlags::STM_END) {
                    log::info!(
                        "\t\tSTM END (cycle = {}, frequency_division = {})",
                        fpga.stm_cycle(),
                        fpga.stm_frequency_division()
                    );
                    fpga.drives().iter().enumerate().for_each(|(i, d)| {
                        let (duty, phase) = d;
                        log::debug!("\tSTM[{}]:", i);
                        log::debug!(
                            "{}",
                            duty.iter()
                                .zip(phase.iter())
                                .enumerate()
                                .map(|(i, (d, p))| {
                                    format!("\n\t\t{}: duty = {}, phase = {}", i, d.duty, p.phase)
                                })
                                .collect::<Vec<_>>()
                                .join("")
                        );
                    })
                }
            } else if fpga.is_legacy_mode() {
                log::info!("\tNormal Legacy mode");
            } else {
                log::info!("\tNormal mode");
            }
            log::info!(
                "\tSilencer step = {}, cycle={}",
                fpga.silencer_step(),
                fpga.silencer_cycle()
            );
            let (m, freq_div_m) = fpga.modulation();
            log::info!(
                "\tModulation size = {}, frequency_division = {}",
                m.len(),
                freq_div_m
            );
            if m.iter().take(fpga.modulation_cycle()).any(|&d| d != 0) {
                log::debug!("\t\t modulation = {:?}", m);
                if !fpga.is_stm_mode() {
                    let (duty, phase) = fpga.drives()[0];
                    log::debug!(
                        "{}",
                        duty.iter()
                            .zip(phase.iter())
                            .enumerate()
                            .map(|(i, (d, p))| {
                                format!("\n\t{}: duty = {}, phase = {}", i, d.duty, p.phase)
                            })
                            .collect::<Vec<_>>()
                            .join("")
                    );
                }
            } else {
                log::info!("\tWithout output");
            }
        });

        Ok(true)
    }

    fn receive(&mut self, rx: &mut RxDatagram) -> anyhow::Result<bool> {
        log::info!("Receive data");

        self.emulator.read(rx);

        Ok(true)
    }

    fn cycle_ticks(&self) -> u16 {
        0
    }

    fn is_open(&self) -> bool {
        true
    }
}
