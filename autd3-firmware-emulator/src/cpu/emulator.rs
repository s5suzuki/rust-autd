/*
 * File: emulator.rs
 * Project: src
 * Created Date: 06/05/2022
 * Author: Shun Suzuki
 * -----
 * Last Modified: 17/05/2022
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2022 Hapis Lab. All rights reserved.
 *
 */

use autd3_driver::{
    Body, CPUControlFlags, FPGAControlFlags, GlobalHeader, MSG_CLEAR, MSG_END, MSG_RD_CPU_VERSION,
    MSG_RD_FPGA_FUNCTION, MSG_RD_FPGA_VERSION, NUM_TRANS_IN_UNIT,
};

use crate::fpga::emulator::FPGAEmulator;

use super::params::*;

pub struct CPUEmulator {
    id: usize,
    pub(crate) msg_id: u8,
    pub(crate) ack: u8,
    mod_cycle: u32,
    stm_cycle: u32,
    pub(crate) fpga: FPGAEmulator,
}

impl CPUEmulator {
    pub fn new(id: usize) -> Self {
        let mut s = Self {
            id,
            msg_id: 0x00,
            ack: 0x0000,
            mod_cycle: 0,
            stm_cycle: 0,
            fpga: FPGAEmulator::new(),
        };
        s.init();
        s
    }

    pub fn id(&self) -> usize {
        self.id
    }

    pub fn fpga(&self) -> &FPGAEmulator {
        &self.fpga
    }

    pub fn send(&mut self, header: &GlobalHeader, body: &Body) {
        self.ecat_recv(header, body)
    }
}

impl CPUEmulator {
    pub(crate) fn init(&mut self) {
        self.fpga.init();
        self.clear();
    }

    fn get_addr(select: u8, addr: u16) -> u16 {
        ((select as u16 & 0x0003) << 14) | (addr & 0x3FFF)
    }

    fn bram_read(&self, select: u8, addr: u16) -> u16 {
        let addr = Self::get_addr(select, addr);
        self.fpga.read(addr)
    }

    fn bram_write(&mut self, select: u8, addr: u16, data: u16) {
        let addr = Self::get_addr(select, addr);
        self.fpga.write(addr, data)
    }

    fn bram_cpy(&mut self, select: u8, addr_base: u16, data: *const u16, size: usize) {
        let mut addr = Self::get_addr(select, addr_base);
        let mut src = data;
        (0..size).for_each(|_| unsafe {
            self.fpga.write(addr, src.read());
            addr += 1;
            src = src.add(1);
        })
    }

    fn bram_set(&mut self, select: u8, addr_base: u16, value: u16, size: usize) {
        let mut addr = Self::get_addr(select, addr_base);
        (0..size).for_each(|_| {
            self.fpga.write(addr, value);
            addr += 1;
        })
    }

    fn synchronize(&mut self, header: &GlobalHeader, body: &Body) {
        let ecat_sync_cycle_ticks = header.sync_header().ecat_sync_cycle_ticks;
        let cycles = body.data;

        self.bram_cpy(
            BRAM_SELECT_CONTROLLER,
            BRAM_ADDR_CYCLE_BASE,
            cycles.as_ptr(),
            NUM_TRANS_IN_UNIT,
        );
        self.bram_write(
            BRAM_SELECT_CONTROLLER,
            BRAM_ADDR_EC_SYNC_CYCLE_TICKS,
            ecat_sync_cycle_ticks,
        );

        // Do nothing to sync
    }

    fn write_mod(&mut self, header: &GlobalHeader) {
        let write = header.size;

        let data = if header.cpu_flag.contains(CPUControlFlags::MOD_BEGIN) {
            self.mod_cycle = 0;
            self.bram_write(BRAM_SELECT_CONTROLLER, BRAM_ADDR_MOD_ADDR_OFFSET, 0);
            let freq_div = header.mod_head().freq_div;
            self.bram_cpy(
                BRAM_SELECT_CONTROLLER,
                BRAM_ADDR_MOD_FREQ_DIV_0,
                &freq_div as *const _ as _,
                std::mem::size_of::<u32>() >> 1,
            );
            header.mod_head().data[..].as_ptr() as *const u16
        } else {
            header.mod_body().data[..].as_ptr() as *const u16
        };

        let segment_capacity =
            (self.mod_cycle & !MOD_BUF_SEGMENT_SIZE_MASK) + MOD_BUF_SEGMENT_SIZE - self.mod_cycle;

        if write as u32 <= segment_capacity {
            self.bram_cpy(
                BRAM_SELECT_MOD,
                ((self.mod_cycle & MOD_BUF_SEGMENT_SIZE_MASK) >> 1) as u16,
                data,
                ((write + 1) >> 1) as usize,
            );
            self.mod_cycle += write as u32;
        } else {
            self.bram_cpy(
                BRAM_SELECT_MOD,
                ((self.mod_cycle & MOD_BUF_SEGMENT_SIZE_MASK) >> 1) as u16,
                data,
                (segment_capacity >> 1) as usize,
            );
            self.mod_cycle += segment_capacity;
            let data = unsafe { data.add(segment_capacity as _) };
            self.bram_write(
                BRAM_SELECT_CONTROLLER,
                BRAM_ADDR_MOD_ADDR_OFFSET,
                ((self.mod_cycle & !MOD_BUF_SEGMENT_SIZE_MASK) >> MOD_BUF_SEGMENT_SIZE_WIDTH)
                    as u16,
            );
            self.bram_cpy(
                BRAM_SELECT_MOD,
                ((self.mod_cycle & MOD_BUF_SEGMENT_SIZE_MASK) >> 1) as _,
                data,
                ((write as u32 - segment_capacity + 1) >> 1) as _,
            );
            self.mod_cycle += write as u32 - segment_capacity;
        }

        if header.cpu_flag.contains(CPUControlFlags::MOD_END) {
            self.bram_write(
                BRAM_SELECT_CONTROLLER,
                BRAM_ADDR_MOD_CYCLE,
                (self.mod_cycle.max(1) - 1) as _,
            );
        }
    }

    fn config_silencer(&mut self, header: &GlobalHeader) {
        let step = header.silencer_header().step;
        let cycle = header.silencer_header().cycle;
        self.bram_write(BRAM_SELECT_CONTROLLER, BRAM_ADDR_SILENT_STEP, step);
        self.bram_write(BRAM_SELECT_CONTROLLER, BRAM_ADDR_SILENT_CYCLE, cycle);
    }

    fn write_normal_op(&mut self, header: &GlobalHeader, body: &Body) {
        if header.fpga_flag.contains(FPGAControlFlags::LEGACY_MODE) {
            (0..NUM_TRANS_IN_UNIT)
                .for_each(|i| self.bram_write(BRAM_SELECT_NORMAL, (i << 1) as _, body.data[i]));
        } else if header.cpu_flag.contains(CPUControlFlags::IS_DUTY) {
            (0..NUM_TRANS_IN_UNIT).for_each(|i| {
                self.bram_write(BRAM_SELECT_NORMAL, (i << 1) as u16 + 1, body.data[i])
            });
        } else {
            (0..NUM_TRANS_IN_UNIT)
                .for_each(|i| self.bram_write(BRAM_SELECT_NORMAL, (i << 1) as u16, body.data[i]));
        }
    }

    fn write_point_stm(&mut self, header: &GlobalHeader, body: &Body) {
        let size: u32;

        let mut src = if header.cpu_flag.contains(CPUControlFlags::STM_BEGIN) {
            self.stm_cycle = 0;
            self.bram_write(BRAM_SELECT_CONTROLLER, BRAM_ADDR_STM_ADDR_OFFSET, 0);
            size = body.point_stm_head().data()[0] as _;
            let freq_div = ((body.point_stm_head().data()[2] as u32) << 16)
                | body.point_stm_head().data()[1] as u32;
            let sound_speed = ((body.point_stm_head().data()[4] as u32) << 16)
                | body.point_stm_head().data()[3] as u32;

            self.bram_cpy(
                BRAM_SELECT_CONTROLLER,
                BRAM_ADDR_STM_FREQ_DIV_0,
                &freq_div as *const _ as _,
                std::mem::size_of::<u32>() >> 1,
            );
            self.bram_cpy(
                BRAM_SELECT_CONTROLLER,
                BRAM_ADDR_SOUND_SPEED_0,
                &sound_speed as *const _ as _,
                std::mem::size_of::<u32>() >> 1,
            );
            unsafe { body.point_stm_head().data().as_ptr().add(5) }
        } else {
            size = body.point_stm_body().data()[0] as _;
            unsafe { body.point_stm_body().data().as_ptr().add(1) }
        };

        let segment_capacity = (self.stm_cycle & !POINT_STM_BUF_SEGMENT_SIZE_MASK)
            + POINT_STM_BUF_SEGMENT_SIZE
            - self.stm_cycle;
        if size <= segment_capacity {
            let mut dst = ((self.stm_cycle & POINT_STM_BUF_SEGMENT_SIZE_MASK) << 3) as u16;
            (0..size as usize).for_each(|_| unsafe {
                self.bram_write(BRAM_SELECT_STM, dst, src.read());
                dst += 1;
                src = src.add(1);
                self.bram_write(BRAM_SELECT_STM, dst, src.read());
                dst += 1;
                src = src.add(1);
                self.bram_write(BRAM_SELECT_STM, dst, src.read());
                dst += 1;
                src = src.add(1);
                self.bram_write(BRAM_SELECT_STM, dst, src.read());
                dst += 1;
                src = src.add(1);
                dst += 4;
            });
            self.stm_cycle += size;
        } else {
            let mut dst = ((self.stm_cycle & POINT_STM_BUF_SEGMENT_SIZE_MASK) << 3) as u16;
            (0..segment_capacity as usize).for_each(|_| unsafe {
                self.bram_write(BRAM_SELECT_STM, dst, src.read());
                dst += 1;
                src = src.add(1);
                self.bram_write(BRAM_SELECT_STM, dst, src.read());
                dst += 1;
                src = src.add(1);
                self.bram_write(BRAM_SELECT_STM, dst, src.read());
                dst += 1;
                src = src.add(1);
                self.bram_write(BRAM_SELECT_STM, dst, src.read());
                dst += 1;
                src = src.add(1);
                dst += 4;
            });
            self.stm_cycle += segment_capacity;

            self.bram_write(
                BRAM_SELECT_CONTROLLER,
                BRAM_ADDR_STM_ADDR_OFFSET,
                ((self.stm_cycle & !POINT_STM_BUF_SEGMENT_SIZE_MASK)
                    >> POINT_STM_BUF_SEGMENT_SIZE_WIDTH) as _,
            );

            let mut dst = ((self.stm_cycle & POINT_STM_BUF_SEGMENT_SIZE_MASK) << 3) as u16;
            let cnt = size - segment_capacity;
            (0..cnt as usize).for_each(|_| unsafe {
                self.bram_write(BRAM_SELECT_STM, dst, src.read());
                dst += 1;
                src = src.add(1);
                self.bram_write(BRAM_SELECT_STM, dst, src.read());
                dst += 1;
                src = src.add(1);
                self.bram_write(BRAM_SELECT_STM, dst, src.read());
                dst += 1;
                src = src.add(1);
                self.bram_write(BRAM_SELECT_STM, dst, src.read());
                dst += 1;
                src = src.add(1);
                dst += 4;
            });
            self.stm_cycle += size - segment_capacity;
        }

        if header.cpu_flag.contains(CPUControlFlags::STM_END) {
            self.bram_write(
                BRAM_SELECT_CONTROLLER,
                BRAM_ADDR_STM_CYCLE,
                (self.stm_cycle.max(1) - 1) as _,
            );
        }
    }

    fn write_gain_stm(&mut self, header: &GlobalHeader, body: &Body) {
        if header.cpu_flag.contains(CPUControlFlags::STM_BEGIN) {
            self.stm_cycle = 0;
            self.bram_write(BRAM_SELECT_CONTROLLER, BRAM_ADDR_STM_ADDR_OFFSET, 0);
            let freq_div = ((body.gain_stm_head().data()[1] as u32) << 16)
                | body.gain_stm_head().data()[0] as u32;
            self.bram_cpy(
                BRAM_SELECT_CONTROLLER,
                BRAM_ADDR_STM_FREQ_DIV_0,
                &freq_div as *const _ as _,
                std::mem::size_of::<u32>() >> 1,
            );
            return;
        }

        let mut src = body.gain_stm_body().data().as_ptr();

        let mut dst = ((self.stm_cycle & GAIN_STM_BUF_SEGMENT_SIZE_MASK) << 9) as u16;
        if header.fpga_flag.contains(FPGAControlFlags::LEGACY_MODE) {
            self.stm_cycle += 1;
        } else if header.cpu_flag.contains(CPUControlFlags::IS_DUTY) {
            dst += 1;
            self.stm_cycle += 1;
        }
        (0..NUM_TRANS_IN_UNIT).for_each(|_| unsafe {
            self.bram_write(BRAM_SELECT_STM, dst, src.read());
            dst += 2;
            src = src.add(1);
        });

        if self.stm_cycle & GAIN_STM_BUF_SEGMENT_SIZE_MASK == 0 {
            self.bram_write(
                BRAM_SELECT_CONTROLLER,
                BRAM_ADDR_STM_ADDR_OFFSET,
                ((self.stm_cycle & !GAIN_STM_BUF_SEGMENT_SIZE_MASK)
                    >> GAIN_STM_BUF_SEGMENT_SIZE_WIDTH) as _,
            );
        }

        if header.cpu_flag.contains(CPUControlFlags::STM_END) {
            self.bram_write(
                BRAM_SELECT_CONTROLLER,
                BRAM_ADDR_STM_CYCLE,
                (self.stm_cycle.max(1) - 1) as _,
            );
        }
    }

    fn get_cpu_version(&self) -> u16 {
        CPU_VERSION
    }

    fn get_fpga_version(&self) -> u16 {
        self.bram_read(BRAM_SELECT_CONTROLLER, BRAM_ADDR_VERSION_NUM)
    }

    fn read_fpga_info(&self) -> u16 {
        self.bram_read(BRAM_SELECT_CONTROLLER, BRAM_ADDR_FPGA_INFO)
    }

    fn clear(&mut self) {
        let freq_div_4k = 40960;

        let ctl_reg = FPGAControlFlags::LEGACY_MODE;
        self.bram_write(
            BRAM_SELECT_CONTROLLER,
            BRAM_ADDR_CTL_REG,
            ctl_reg.bits() as _,
        );

        self.bram_write(BRAM_SELECT_CONTROLLER, BRAM_ADDR_SILENT_STEP, 10);
        self.bram_write(BRAM_SELECT_CONTROLLER, BRAM_ADDR_SILENT_CYCLE, 4096);

        self.stm_cycle = 0;

        self.mod_cycle = 2;
        self.bram_write(
            BRAM_SELECT_CONTROLLER,
            BRAM_ADDR_MOD_CYCLE,
            (self.mod_cycle.max(1) - 1) as _,
        );
        self.bram_cpy(
            BRAM_SELECT_CONTROLLER,
            BRAM_ADDR_MOD_FREQ_DIV_0,
            &freq_div_4k as *const _ as _,
            std::mem::size_of::<u32>() >> 1,
        );
        self.bram_write(BRAM_SELECT_MOD, 0, 0x0000);

        self.bram_set(BRAM_SELECT_NORMAL, 0, 0x0000, NUM_TRANS_IN_UNIT << 1);
    }

    fn ecat_recv(&mut self, header: &GlobalHeader, body: &Body) {
        if self.msg_id == header.msg_id {
            return;
        }

        self.msg_id = header.msg_id;
        let read_fpga_info = header.cpu_flag.contains(CPUControlFlags::READS_FPGA_INFO);
        if read_fpga_info {
            self.ack = self.read_fpga_info() as _;
        }

        match self.msg_id {
            MSG_CLEAR => {
                self.clear();
            }
            MSG_RD_CPU_VERSION => {
                self.ack = (self.get_cpu_version() & 0xFF) as _;
            }
            MSG_RD_FPGA_VERSION => {
                self.ack = (self.get_fpga_version() & 0xFF) as _;
            }
            MSG_RD_FPGA_FUNCTION => {
                self.ack = ((self.get_fpga_version() >> 8) & 0xFF) as _;
            }
            _ => {
                if self.msg_id > MSG_END {
                    return;
                }

                let ctl_reg = header.fpga_flag;
                self.bram_write(
                    BRAM_SELECT_CONTROLLER,
                    BRAM_ADDR_CTL_REG,
                    ctl_reg.bits() as _,
                );

                if header.cpu_flag.contains(CPUControlFlags::MOD) {
                    self.write_mod(header);
                } else if header.cpu_flag.contains(CPUControlFlags::CONFIG_SILENCER) {
                    self.config_silencer(header);
                } else if header.cpu_flag.contains(CPUControlFlags::CONFIG_SYNC) {
                    self.synchronize(header, body);
                    return;
                }

                if !header.cpu_flag.contains(CPUControlFlags::WRITE_BODY) {
                    return;
                }

                if !ctl_reg.contains(FPGAControlFlags::STM_MODE) {
                    self.write_normal_op(header, body);
                    return;
                }

                if !ctl_reg.contains(FPGAControlFlags::STM_GAIN_MODE) {
                    self.write_point_stm(header, body);
                } else {
                    self.write_gain_stm(header, body);
                }
            }
        }
    }
}
