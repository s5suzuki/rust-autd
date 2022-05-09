/*
 * File: fpga_emulator.rs
 * Project: src
 * Created Date: 06/05/2022
 * Author: Shun Suzuki
 * -----
 * Last Modified: 08/05/2022
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2022 Hapis Lab. All rights reserved.
 *
 */

use num_integer::Roots;

use autd3_driver::{Duty, Phase, NUM_TRANS_IN_UNIT};

use super::params::*;

const TR_POS: [u32; NUM_TRANS_IN_UNIT] = [
    0x00000000, 0x01960000, 0x032c0000, 0x04c30000, 0x06590000, 0x07ef0000, 0x09860000, 0x0b1c0000,
    0x0cb30000, 0x0e490000, 0x0fdf0000, 0x11760000, 0x130c0000, 0x14a30000, 0x16390000, 0x17d00000,
    0x19660000, 0x1afc0000, 0x00000196, 0x04c30196, 0x06590196, 0x07ef0196, 0x09860196, 0x0b1c0196,
    0x0cb30196, 0x0e490196, 0x0fdf0196, 0x11760196, 0x130c0196, 0x14a30196, 0x16390196, 0x17d00196,
    0x1afc0196, 0x0000032c, 0x0196032c, 0x032c032c, 0x04c3032c, 0x0659032c, 0x07ef032c, 0x0986032c,
    0x0b1c032c, 0x0cb3032c, 0x0e49032c, 0x0fdf032c, 0x1176032c, 0x130c032c, 0x14a3032c, 0x1639032c,
    0x17d0032c, 0x1966032c, 0x1afc032c, 0x000004c3, 0x019604c3, 0x032c04c3, 0x04c304c3, 0x065904c3,
    0x07ef04c3, 0x098604c3, 0x0b1c04c3, 0x0cb304c3, 0x0e4904c3, 0x0fdf04c3, 0x117604c3, 0x130c04c3,
    0x14a304c3, 0x163904c3, 0x17d004c3, 0x196604c3, 0x1afc04c3, 0x00000659, 0x01960659, 0x032c0659,
    0x04c30659, 0x06590659, 0x07ef0659, 0x09860659, 0x0b1c0659, 0x0cb30659, 0x0e490659, 0x0fdf0659,
    0x11760659, 0x130c0659, 0x14a30659, 0x16390659, 0x17d00659, 0x19660659, 0x1afc0659, 0x000007ef,
    0x019607ef, 0x032c07ef, 0x04c307ef, 0x065907ef, 0x07ef07ef, 0x098607ef, 0x0b1c07ef, 0x0cb307ef,
    0x0e4907ef, 0x0fdf07ef, 0x117607ef, 0x130c07ef, 0x14a307ef, 0x163907ef, 0x17d007ef, 0x196607ef,
    0x1afc07ef, 0x00000986, 0x01960986, 0x032c0986, 0x04c30986, 0x06590986, 0x07ef0986, 0x09860986,
    0x0b1c0986, 0x0cb30986, 0x0e490986, 0x0fdf0986, 0x11760986, 0x130c0986, 0x14a30986, 0x16390986,
    0x17d00986, 0x19660986, 0x1afc0986, 0x00000b1c, 0x01960b1c, 0x032c0b1c, 0x04c30b1c, 0x06590b1c,
    0x07ef0b1c, 0x09860b1c, 0x0b1c0b1c, 0x0cb30b1c, 0x0e490b1c, 0x0fdf0b1c, 0x11760b1c, 0x130c0b1c,
    0x14a30b1c, 0x16390b1c, 0x17d00b1c, 0x19660b1c, 0x1afc0b1c, 0x00000cb3, 0x01960cb3, 0x032c0cb3,
    0x04c30cb3, 0x06590cb3, 0x07ef0cb3, 0x09860cb3, 0x0b1c0cb3, 0x0cb30cb3, 0x0e490cb3, 0x0fdf0cb3,
    0x11760cb3, 0x130c0cb3, 0x14a30cb3, 0x16390cb3, 0x17d00cb3, 0x19660cb3, 0x1afc0cb3, 0x00000e49,
    0x01960e49, 0x032c0e49, 0x04c30e49, 0x06590e49, 0x07ef0e49, 0x09860e49, 0x0b1c0e49, 0x0cb30e49,
    0x0e490e49, 0x0fdf0e49, 0x11760e49, 0x130c0e49, 0x14a30e49, 0x16390e49, 0x17d00e49, 0x19660e49,
    0x1afc0e49, 0x00000fdf, 0x01960fdf, 0x032c0fdf, 0x04c30fdf, 0x06590fdf, 0x07ef0fdf, 0x09860fdf,
    0x0b1c0fdf, 0x0cb30fdf, 0x0e490fdf, 0x0fdf0fdf, 0x11760fdf, 0x130c0fdf, 0x14a30fdf, 0x16390fdf,
    0x17d00fdf, 0x19660fdf, 0x1afc0fdf, 0x00001176, 0x01961176, 0x032c1176, 0x04c31176, 0x06591176,
    0x07ef1176, 0x09861176, 0x0b1c1176, 0x0cb31176, 0x0e491176, 0x0fdf1176, 0x11761176, 0x130c1176,
    0x14a31176, 0x16391176, 0x17d01176, 0x19661176, 0x1afc1176, 0x0000130c, 0x0196130c, 0x032c130c,
    0x04c3130c, 0x0659130c, 0x07ef130c, 0x0986130c, 0x0b1c130c, 0x0cb3130c, 0x0e49130c, 0x0fdf130c,
    0x1176130c, 0x130c130c, 0x14a3130c, 0x1639130c, 0x17d0130c, 0x1966130c, 0x1afc130c, 0x000014a3,
    0x019614a3, 0x032c14a3, 0x04c314a3, 0x065914a3, 0x07ef14a3, 0x098614a3, 0x0b1c14a3, 0x0cb314a3,
    0x0e4914a3, 0x0fdf14a3, 0x117614a3, 0x130c14a3, 0x14a314a3, 0x163914a3, 0x17d014a3, 0x196614a3,
    0x1afc14a3,
];

pub struct FPGAEmulator {
    controller_bram: Vec<u16>,
    modulator_bram: Vec<u16>,
    normal_op_bram: Vec<u16>,
    stm_op_bram: Vec<u16>,
}

impl FPGAEmulator {
    pub(crate) fn new() -> Self {
        Self {
            controller_bram: vec![0x0000; 512],
            modulator_bram: vec![0x0000; 32768],
            normal_op_bram: vec![0x0000; 512],
            stm_op_bram: vec![0x0000; 524288],
        }
    }

    pub(crate) fn init(&mut self) {
        self.controller_bram[ADDR_VERSION_NUM] =
            (ENABLED_FEATURES_BITS as u16) << 8 | VERSION_NUM as u16;
    }

    pub(crate) fn read(&self, addr: u16) -> u16 {
        let select = (addr >> 14) & 0x0003;
        let addr = (addr & 0x3FFF) as usize;
        match select {
            BRAM_SELECT_CONTROLLER => self.controller_bram[addr],
            BRAM_SELECT_MOD => {
                let offset = self.controller_bram[ADDR_MOD_ADDR_OFFSET];
                let addr = (offset as usize) << 14 | addr;
                self.modulator_bram[addr]
            }
            BRAM_SELECT_NORMAL => self.normal_op_bram[addr],
            BRAM_SELECT_STM => {
                let offset = self.controller_bram[ADDR_STM_ADDR_OFFSET];
                let addr = (offset as usize) << 14 | addr;
                self.stm_op_bram[addr]
            }
            _ => unreachable!(),
        }
    }

    pub(crate) fn write(&mut self, addr: u16, data: u16) {
        let select = (addr >> 14) & 0x0003;
        let addr = (addr & 0x3FFF) as usize;
        match select {
            BRAM_SELECT_CONTROLLER => self.controller_bram[addr] = data,
            BRAM_SELECT_MOD => {
                let offset = self.controller_bram[ADDR_MOD_ADDR_OFFSET];
                let addr = (offset as usize) << 14 | addr;
                self.modulator_bram[addr] = data;
            }
            BRAM_SELECT_NORMAL => self.normal_op_bram[addr] = data,
            BRAM_SELECT_STM => {
                let offset = self.controller_bram[ADDR_STM_ADDR_OFFSET];
                let addr = (offset as usize) << 14 | addr;
                self.stm_op_bram[addr] = data
            }
            _ => unreachable!(),
        }
    }

    pub fn is_legacy_mode(&self) -> bool {
        (self.controller_bram[ADDR_CTL_REG] & (1 << CTL_REG_LEGACY_MODE_BIT)) != 0
    }

    pub fn is_force_fan(&self) -> bool {
        (self.controller_bram[ADDR_CTL_REG] & (1 << CTL_REG_FORCE_FAN_BIT)) != 0
    }

    pub fn is_stm_mode(&self) -> bool {
        (self.controller_bram[ADDR_CTL_REG] & (1 << CTL_REG_OP_MODE_BIT)) != 0
    }

    pub fn is_stm_gain_mode(&self) -> bool {
        (self.controller_bram[ADDR_CTL_REG] & (1 << CTL_REG_STM_GAIN_MODE_BIT)) != 0
    }

    pub fn cycle_ticks(&self) -> u16 {
        self.controller_bram[ADDR_EC_SYNC_CYCLE_TICKS]
    }

    pub fn silencer_cycle(&self) -> u16 {
        self.controller_bram[ADDR_SILENT_CYCLE]
    }

    pub fn silencer_step(&self) -> u16 {
        self.controller_bram[ADDR_SILENT_STEP]
    }

    pub fn cycles(&self) -> [u16; NUM_TRANS_IN_UNIT] {
        self.controller_bram[ADDR_CYCLE_BASE..]
            .iter()
            .take(NUM_TRANS_IN_UNIT).copied()
            .collect::<Vec<u16>>()
            .try_into()
            .unwrap()
    }

    pub fn stm_frequency_division(&self) -> u32 {
        ((self.controller_bram[ADDR_STM_FREQ_DIV_1] as u32) << 16) & 0xFFFF0000
            | self.controller_bram[ADDR_STM_FREQ_DIV_0] as u32 & 0x0000FFFF
    }

    pub fn stm_cycle(&self) -> usize {
        self.controller_bram[ADDR_STM_CYCLE] as usize + 1
    }

    pub fn sound_speed(&self) -> u32 {
        ((self.controller_bram[ADDR_SOUND_SPEED_1] as u32) << 16) & 0xFFFF0000
            | self.controller_bram[ADDR_SOUND_SPEED_0] as u32 & 0x0000FFFF
    }

    pub fn modulation_frequency_division(&self) -> u32 {
        ((self.controller_bram[ADDR_MOD_FREQ_DIV_1] as u32) << 16) & 0xFFFF0000
            | self.controller_bram[ADDR_MOD_FREQ_DIV_0] as u32 & 0x0000FFFF
    }

    pub fn modulation_cycle(&self) -> usize {
        self.controller_bram[ADDR_MOD_CYCLE] as usize + 1
    }

    pub fn modulation(&self) -> (Vec<u8>, u32) {
        let cycle = self.modulation_cycle();
        let mut m = Vec::with_capacity(cycle);

        (0..cycle >> 1).for_each(|i| {
            let b = self.modulator_bram[i];
            m.push((b & 0x00FF) as u8);
            m.push(((b >> 8) & 0x00FF) as u8);
        });
        if cycle % 2 != 0 {
            let b = self.modulator_bram[(cycle + 1) >> 1];
            m.push((b & 0x00FF) as u8);
        }

        (m, self.modulation_frequency_division())
    }

    fn normal_drive(&self) -> ([Duty; NUM_TRANS_IN_UNIT], [Phase; NUM_TRANS_IN_UNIT]) {
        (
            self.normal_op_bram
                .iter()
                .skip(1)
                .step_by(2)
                .take(NUM_TRANS_IN_UNIT)
                .map(|d| Duty { duty: *d })
                .collect::<Vec<_>>()
                .try_into()
                .unwrap(),
            self.normal_op_bram
                .iter()
                .step_by(2)
                .take(NUM_TRANS_IN_UNIT)
                .map(|d| Phase { phase: *d })
                .collect::<Vec<_>>()
                .try_into()
                .unwrap(),
        )
    }

    fn legacy_drive(&self) -> ([Duty; NUM_TRANS_IN_UNIT], [Phase; NUM_TRANS_IN_UNIT]) {
        (
            self.normal_op_bram
                .iter()
                .step_by(2)
                .take(NUM_TRANS_IN_UNIT)
                .map(|d| {
                    let duty = (d >> 8) & 0xFF;
                    let duty = ((duty << 3) | 0x07) + 1;
                    Duty { duty }
                })
                .collect::<Vec<_>>()
                .try_into()
                .unwrap(),
            self.normal_op_bram
                .iter()
                .step_by(2)
                .take(NUM_TRANS_IN_UNIT)
                .map(|d| {
                    let phase = d & 0xFF;
                    let phase = phase << 4;
                    Phase { phase }
                })
                .collect::<Vec<_>>()
                .try_into()
                .unwrap(),
        )
    }

    fn gain_stm_normal_drives(
        &self,
    ) -> Vec<([Duty; NUM_TRANS_IN_UNIT], [Phase; NUM_TRANS_IN_UNIT])> {
        let cycle = self.stm_cycle();
        self.stm_op_bram
            .chunks(512)
            .take(cycle)
            .map(|d| {
                (
                    d.iter()
                        .skip(1)
                        .step_by(2)
                        .take(NUM_TRANS_IN_UNIT)
                        .map(|d| Duty { duty: *d })
                        .collect::<Vec<_>>()
                        .try_into()
                        .unwrap(),
                    d.iter()
                        .step_by(2)
                        .take(NUM_TRANS_IN_UNIT)
                        .map(|d| Phase { phase: *d })
                        .collect::<Vec<_>>()
                        .try_into()
                        .unwrap(),
                )
            })
            .collect()
    }

    fn gain_stm_legacy_drives(
        &self,
    ) -> Vec<([Duty; NUM_TRANS_IN_UNIT], [Phase; NUM_TRANS_IN_UNIT])> {
        let cycle = self.stm_cycle();
        self.stm_op_bram
            .chunks(512)
            .take(cycle)
            .map(|d| {
                (
                    d.iter()
                        .step_by(2)
                        .take(NUM_TRANS_IN_UNIT)
                        .map(|d| {
                            let duty = (d >> 8) & 0xFF;
                            let duty = ((duty << 3) | 0x07) + 1;
                            Duty { duty }
                        })
                        .collect::<Vec<_>>()
                        .try_into()
                        .unwrap(),
                    d.iter()
                        .step_by(2)
                        .take(NUM_TRANS_IN_UNIT)
                        .map(|d| {
                            let phase = d & 0xFF;
                            let phase = phase << 4;
                            Phase { phase }
                        })
                        .collect::<Vec<_>>()
                        .try_into()
                        .unwrap(),
                )
            })
            .collect()
    }

    pub fn point_stm_drives(&self) -> Vec<([Duty; NUM_TRANS_IN_UNIT], [Phase; NUM_TRANS_IN_UNIT])> {
        let cycle = self.stm_cycle();
        let ultrasound_cycles = self.cycles();
        let sound_speed = self.sound_speed() as u64;
        self.stm_op_bram
            .chunks(8)
            .take(cycle)
            .map(|d| {
                let x = (((d[1] as u32) << 16) & 0x30000) | d[0] as u32;
                let x = if (x & 0x20000) == 0 {
                    x as i32
                } else {
                    let a = x & 0x1FFFF;
                    -131072 + a as i32
                };
                let y = (((d[2] as u32) << 14) & 0x3C000) | (((d[1] as u32) >> 2) & 0x3FFFF);
                let y = if (y & 0x20000) == 0 {
                    y as i32
                } else {
                    let a = y & 0x1FFFF;
                    -131072 + a as i32
                };
                let z = (((d[3] as u32) << 12) & 0x3F000) | (((d[2] as u32) >> 4) & 0xFFF);
                let z = if (z & 0x20000) == 0 {
                    z as i32
                } else {
                    let a = z & 0x1FFFF;
                    -131072 + a as i32
                };
                let duty_shift = (d[3] >> 6) & 0xF;

                (
                    ultrasound_cycles
                        .iter()
                        .map(|c| Duty {
                            duty: c >> (duty_shift + 1),
                        })
                        .collect::<Vec<_>>()
                        .try_into()
                        .unwrap(),
                    (0..NUM_TRANS_IN_UNIT)
                        .map(|i| {
                            let tr_x = ((TR_POS[i] >> 16) & 0xFFFF) as i32;
                            let tr_y = (TR_POS[i] & 0xFFFF) as i32;
                            let d2 = (x - tr_x) * (x - tr_x) + (y - tr_y) * (y - tr_y) + z * z;
                            let d = (d2 as u32).sqrt() as u64;
                            let q = (d << 22) / sound_speed;
                            let p = q % ultrasound_cycles[i] as u64;
                            Phase { phase: p as _ }
                        })
                        .collect::<Vec<_>>()
                        .try_into()
                        .unwrap(),
                )
            })
            .collect()
    }

    pub fn drives(&self) -> Vec<([Duty; NUM_TRANS_IN_UNIT], [Phase; NUM_TRANS_IN_UNIT])> {
        if self.is_stm_mode() {
            if self.is_stm_gain_mode() {
                if self.is_legacy_mode() {
                    self.gain_stm_legacy_drives()
                } else {
                    self.gain_stm_normal_drives()
                }
            } else {
                self.point_stm_drives()
            }
        } else if self.is_legacy_mode() {
            vec![self.legacy_drive()]
        } else {
            vec![self.normal_drive()]
        }
    }
}

impl Default for FPGAEmulator {
    fn default() -> Self {
        Self::new()
    }
}
