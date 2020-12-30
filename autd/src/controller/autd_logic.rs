/*
 * File: autd_logic.rs
 * Project: controller
 * Created Date: 30/12/2020
 * Author: Shun Suzuki
 * -----
 * Last Modified: 30/12/2020
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2020 Hapis Lab. All rights reserved.
 *
 */

use std::{error::Error, mem::size_of, ptr::copy_nonoverlapping};

use crate::{
    core::{configuration::Configuration, consts::*, *},
    gain::Gain,
    geometry::Geometry,
    link::Link,
    modulation::Modulation,
    sequence::PointSequence,
    Float,
};

use super::{GainPtr, ModPtr};

pub struct AUTDLogic<L: Link> {
    geometry: Geometry,
    link: L,
    rx_data: Vec<u8>,
    seq_mode: bool,
    pub(crate) silent_mode: bool,
    config: Configuration,
}

impl<L: Link> AUTDLogic<L> {
    pub fn geometry(&self) -> &Geometry {
        &self.geometry
    }
}

impl<L: Link> AUTDLogic<L> {
    pub(crate) fn new(geometry: Geometry, link: L) -> Self {
        Self {
            geometry,
            link,
            rx_data: vec![],
            seq_mode: false,
            silent_mode: true,
            config: Configuration::default(),
        }
    }

    pub(crate) fn is_open(&self) -> bool {
        self.link.is_open()
    }

    pub(crate) fn build_gain<G: Gain>(&self, g: &mut G) {
        g.build(&self.geometry)
    }

    pub(crate) fn build_modulation<M: Modulation>(&self, m: &mut M) {
        m.build(self.config)
    }

    pub(crate) fn build_gain_ptr(&self, g: &mut GainPtr) {
        g.build(&self.geometry)
    }

    pub(crate) fn build_modulation_ptr(&self, m: &mut ModPtr) {
        m.build(self.config)
    }

    pub(crate) fn send_gain_mod<G: Gain, M: Modulation>(
        &mut self,
        g: Option<&G>,
        m: Option<&mut M>,
    ) -> Result<u8, Box<dyn Error>> {
        if g.is_some() {
            self.seq_mode = false;
        }

        let dev_num = self.geometry.num_devices();
        let (msg_id, body) = Self::make_body(g, m, dev_num, self.silent_mode, self.seq_mode);
        self.link.send(&body)?;
        Ok(msg_id)
    }

    pub(crate) fn send_gain_mod_ptr(
        &mut self,
        g: Option<GainPtr>,
        m: Option<&mut ModPtr>,
    ) -> Result<u8, Box<dyn Error>> {
        if g.is_some() {
            self.seq_mode = false;
        }

        let dev_num = self.geometry.num_devices();
        let (msg_id, body) = Self::make_body_ptr(g, m, dev_num, self.silent_mode, self.seq_mode);
        self.link.send(&body)?;
        Ok(msg_id)
    }

    pub(crate) fn send_gain_mod_blocking<G: Gain, M: Modulation>(
        &mut self,
        g: Option<&G>,
        m: Option<&mut M>,
    ) -> Result<bool, Box<dyn Error>> {
        let msg_id = self.send_gain_mod(g, m)?;
        let dev_num = self.geometry.num_devices();
        self.wait_msg_processed(dev_num, msg_id, 0xFF, 200)
    }

    pub(crate) fn send_seq_blocking(
        &mut self,
        seq: &mut PointSequence,
    ) -> Result<bool, Box<dyn Error>> {
        self.seq_mode = true;

        let (msg_id, body) = Self::make_seq_body(seq, &self.geometry, self.silent_mode);
        self.link.send(&body)?;

        let dev_num = self.geometry.num_devices();
        if seq.sent() == seq.control_points().len() {
            self.wait_msg_processed(dev_num, 0xC0, 0xE0, 2000)
        } else {
            self.wait_msg_processed(dev_num, msg_id, 0xFF, 200)
        }
    }

    pub(crate) fn send_data(&mut self, data: &[u8]) -> Result<u8, Box<dyn Error>> {
        let msg_id = data[0];
        self.link.send(&data)?;
        Ok(msg_id)
    }

    pub(crate) fn send_data_blocking(&mut self, data: Vec<u8>) -> Result<bool, Box<dyn Error>> {
        let msg_id = self.send_data(&data)?;
        let dev_num = self.geometry.num_devices();
        self.wait_msg_processed(dev_num, msg_id, 0xFF, 200)
    }

    fn send_header_blocking(
        &mut self,
        command: CommandType,
        max_trial: usize,
    ) -> Result<bool, Box<dyn Error>> {
        let header = RxGlobalHeader::new_with_cmd(command);
        let dev_num = self.geometry.num_devices();
        unsafe {
            self.link.send(Self::convert_to_u8_slice(&header))?;
        }
        self.wait_msg_processed(dev_num, command as u8, 0xFF, max_trial)
    }

    pub(crate) fn calibrate(&mut self, config: Configuration) -> Result<bool, Box<dyn Error>> {
        self.config = config;
        self.send_header_blocking(CommandType::CmdInitRefClock, 5000)
    }

    pub(crate) fn calibrate_seq(&mut self) -> Result<(), Box<dyn Error>> {
        let rx_data = &self.rx_data;
        let mut laps = Vec::with_capacity(rx_data.len() / 2);
        for j in 0..laps.capacity() {
            let lap_raw = ((rx_data[2 * j + 1] as u16) << 8) | rx_data[2 * j] as u16;
            laps.push(lap_raw & 0x03FF);
        }
        let minimum = laps.iter().min().unwrap();
        let diffs = laps.iter().map(|&d| d - minimum).collect::<Vec<_>>();
        let diff_max = *diffs.iter().max().unwrap();
        let diffs: Vec<u16> = if diff_max == 0 {
            return Ok(());
        } else if diff_max > 500 {
            let laps = laps
                .iter()
                .map(|&d| if d < 500 { d + 1000 } else { d })
                .collect::<Vec<_>>();
            let minimum = laps.iter().min().unwrap();
            laps.iter().map(|d| d - minimum).collect()
        } else {
            diffs
        };

        let dev_num = diffs.len();
        let calib_body = Self::make_calib_body(diffs);
        self.link.send(&calib_body)?;
        self.wait_msg_processed(dev_num, 0xE0, 0xE0, 200)?;

        Ok(())
    }

    pub(crate) fn clear(&mut self) -> Result<bool, Box<dyn Error>> {
        self.send_header_blocking(CommandType::CmdClear, 5000)
    }

    pub(crate) fn close(&mut self) -> Result<bool, Box<dyn Error>> {
        self.clear()?;
        self.link.close()?;
        Ok(true)
    }

    pub(crate) fn set_delay(&mut self, delays: &[DataArray]) -> Result<(), Box<dyn Error>> {
        let dev_num = self.geometry.num_devices();
        let size = size_of::<RxGlobalHeader>() + dev_num * 2 * NUM_TRANS_IN_UNIT;
        let mut body = vec![0x00; size];

        unsafe {
            let header = RxGlobalHeader::new_with_cmd(CommandType::SetDelay);
            let src_ptr = &header as *const RxGlobalHeader as *const u8;
            let dst_ptr = body.as_mut_ptr();
            copy_nonoverlapping(src_ptr, dst_ptr, size_of::<RxGlobalHeader>());
            header.msg_id
        };

        let mut cursor = size_of::<RxGlobalHeader>();
        let byte_size = NUM_TRANS_IN_UNIT * 2;
        unsafe {
            for delay in delays {
                let dst_ptr = body.as_mut_ptr().add(cursor);
                copy_nonoverlapping(delay.as_ptr() as *const u8, dst_ptr, byte_size);
                cursor += byte_size;
            }
        }
        self.send_data_blocking(body)?;

        Ok(())
    }

    #[allow(clippy::needless_range_loop)]
    pub(crate) fn firmware_info_list(&mut self) -> Result<Vec<FirmwareInfo>, Box<dyn Error>> {
        let size = self.geometry.num_devices();

        let mut cpu_versions: Vec<u16> = vec![0x0000; size];
        let mut fpga_versions: Vec<u16> = vec![0x0000; size];

        self.send_header_blocking(CommandType::CmdReadCpuVerLsb, 50)?;
        for i in 0..size {
            cpu_versions[i] = self.rx_data[2 * i] as u16;
        }

        self.send_header_blocking(CommandType::CmdReadCpuVerMsb, 50)?;
        for i in 0..size {
            cpu_versions[i] |= (self.rx_data[2 * i] as u16) << 8;
        }

        self.send_header_blocking(CommandType::CmdReadFpgaVerLsb, 50)?;
        for i in 0..size {
            fpga_versions[i] = self.rx_data[2 * i] as u16;
        }

        self.send_header_blocking(CommandType::CmdReadFpgaVerMsb, 50)?;
        for i in 0..size {
            fpga_versions[i] |= (self.rx_data[2 * i] as u16) << 8;
        }

        let mut res = Vec::with_capacity(size);
        for i in 0..size {
            let firm_info = FirmwareInfo::new(i as u16, cpu_versions[i], fpga_versions[i]);
            res.push(firm_info);
        }

        Ok(res)
    }

    fn make_calib_body(diffs: Vec<u16>) -> Vec<u8> {
        let header = RxGlobalHeader::new_with_cmd(CommandType::CmdCalibSeqClock);
        let mut body =
            vec![0x00; size_of::<RxGlobalHeader>() + NUM_TRANS_IN_UNIT * 2 * diffs.len()];
        unsafe {
            copy_nonoverlapping(
                &header as *const RxGlobalHeader as *const u8,
                body.as_mut_ptr(),
                size_of::<RxGlobalHeader>(),
            );
            let mut cursor = size_of::<RxGlobalHeader>();
            for diff in diffs {
                body[cursor] = (diff & 0x00FF) as u8;
                body[cursor + 1] = ((diff & 0xFF00) >> 8) as u8;
                cursor += NUM_TRANS_IN_UNIT * 2;
            }
        }
        body
    }

    fn wait_msg_processed(
        &mut self,
        dev_num: usize,
        msg_id: u8,
        mask: u8,
        max_trial: usize,
    ) -> Result<bool, Box<dyn Error>> {
        let buffer_len = dev_num * INPUT_FRAME_SIZE;

        self.rx_data.resize(buffer_len, 0x00);
        for _ in 0..max_trial {
            self.link.read(&mut self.rx_data, buffer_len)?;

            let processed = (0..dev_num)
                .map(|dev| self.rx_data[dev as usize * INPUT_FRAME_SIZE + 1])
                .filter(|&proc_id| (proc_id & mask) == msg_id)
                .count();

            if processed == dev_num {
                return Ok(true);
            }

            let wait_t = (EC_TRAFFIC_DELAY * 1000.0 / EC_DEVICE_PER_FRAME as Float
                * dev_num as Float)
                .ceil() as u64;
            let wait_t = 1.max(wait_t);
            std::thread::sleep(std::time::Duration::from_millis(wait_t));
        }
        Ok(false)
    }

    fn make_body<G: Gain, M: Modulation>(
        gain: Option<&G>,
        m: Option<&mut M>,
        num_devices: usize,
        is_silent: bool,
        is_seq_mode: bool,
    ) -> (u8, Vec<u8>) {
        let num_bodies = if gain.is_some() { num_devices } else { 0 };
        let size = size_of::<RxGlobalHeader>() + NUM_TRANS_IN_UNIT * 2 * num_bodies;

        let mut body = vec![0x00; size];
        let mut ctrl_flags = RxGlobalControlFlags::NONE;
        if is_silent {
            ctrl_flags |= RxGlobalControlFlags::SILENT;
        }
        if is_seq_mode {
            ctrl_flags |= RxGlobalControlFlags::SEQ_MODE;
        }

        let mod_data = if let Some(modulation) = m {
            let sent = *modulation.sent();
            let mod_size = num::clamp(modulation.buffer().len() - sent, 0, MOD_FRAME_SIZE);
            if sent == 0 {
                ctrl_flags |= RxGlobalControlFlags::LOOP_BEGIN;
            }
            if sent + mod_size >= modulation.buffer().len() {
                ctrl_flags |= RxGlobalControlFlags::LOOP_END;
            }
            *modulation.sent() += mod_size;
            &modulation.buffer()[sent..(sent + mod_size)]
        } else {
            &[]
        };
        let msg_id = unsafe {
            let header = RxGlobalHeader::new_op(ctrl_flags, mod_data);
            let src_ptr = &header as *const RxGlobalHeader as *const u8;
            let dst_ptr = body.as_mut_ptr();
            copy_nonoverlapping(src_ptr, dst_ptr, size_of::<RxGlobalHeader>());
            header.msg_id
        };

        if let Some(gain) = gain {
            let mut cursor = size_of::<RxGlobalHeader>();
            let byte_size = NUM_TRANS_IN_UNIT * 2;
            let gain_ptr = gain.get_data().as_ptr();
            unsafe {
                for i in 0..num_devices {
                    let src_ptr = gain_ptr.add(i * byte_size);
                    let dst_ptr = body.as_mut_ptr().add(cursor);

                    copy_nonoverlapping(src_ptr, dst_ptr, byte_size);
                    cursor += byte_size;
                }
            }
        }

        (msg_id, body)
    }

    pub(crate) fn make_body_ptr(
        gain: Option<GainPtr>,
        m: Option<&mut ModPtr>,
        num_devices: usize,
        is_silent: bool,
        is_seq_mode: bool,
    ) -> (u8, Vec<u8>) {
        let num_bodies = if gain.is_some() { num_devices } else { 0 };
        let size = size_of::<RxGlobalHeader>() + NUM_TRANS_IN_UNIT * 2 * num_bodies;

        let mut body = vec![0x00; size];
        let mut ctrl_flags = RxGlobalControlFlags::NONE;
        if is_silent {
            ctrl_flags |= RxGlobalControlFlags::SILENT;
        }
        if is_seq_mode {
            ctrl_flags |= RxGlobalControlFlags::SEQ_MODE;
        }

        let mod_data = if let Some(modulation) = m {
            let sent = *modulation.sent();
            let mod_size = num::clamp(modulation.buffer().len() - sent, 0, MOD_FRAME_SIZE);
            if sent == 0 {
                ctrl_flags |= RxGlobalControlFlags::LOOP_BEGIN;
            }
            if sent + mod_size >= modulation.buffer().len() {
                ctrl_flags |= RxGlobalControlFlags::LOOP_END;
            }
            *modulation.sent() += mod_size;
            &modulation.buffer()[sent..(sent + mod_size)]
        } else {
            &[]
        };
        let msg_id = unsafe {
            let header = RxGlobalHeader::new_op(ctrl_flags, mod_data);
            let src_ptr = &header as *const RxGlobalHeader as *const u8;
            let dst_ptr = body.as_mut_ptr();
            copy_nonoverlapping(src_ptr, dst_ptr, size_of::<RxGlobalHeader>());
            header.msg_id
        };

        if let Some(gain) = gain {
            let mut cursor = size_of::<RxGlobalHeader>();
            let byte_size = NUM_TRANS_IN_UNIT * 2;
            let gain_ptr = gain.get_data().as_ptr();
            unsafe {
                for i in 0..num_devices {
                    let src_ptr = gain_ptr.add(i * byte_size);
                    let dst_ptr = body.as_mut_ptr().add(cursor);

                    copy_nonoverlapping(src_ptr, dst_ptr, byte_size);
                    cursor += byte_size;
                }
            }
        }
        (msg_id, body)
    }

    fn make_seq_body(
        seq: &mut PointSequence,
        geometry: &Geometry,
        is_silent: bool,
    ) -> (u8, Vec<u8>) {
        let num_devices = geometry.num_devices();
        let size = size_of::<RxGlobalHeader>() + NUM_TRANS_IN_UNIT * 2 * num_devices;

        let mut body = vec![0x00; size];
        let send_size = num::clamp(seq.control_points().len() - seq.sent(), 0, 40);

        let mut ctrl_flags = RxGlobalControlFlags::SEQ_MODE;
        if is_silent {
            ctrl_flags |= RxGlobalControlFlags::SILENT;
        }

        if seq.sent() == 0 {
            ctrl_flags |= RxGlobalControlFlags::SEQ_BEGIN;
        }
        if seq.sent() + send_size >= seq.control_points().len() {
            ctrl_flags |= RxGlobalControlFlags::SEQ_END;
        }
        let msg_id = unsafe {
            let header =
                RxGlobalHeader::new_seq(ctrl_flags, send_size as u16, seq.sampling_freq_div());
            let src_ptr = &header as *const RxGlobalHeader as *const u8;
            let dst_ptr = body.as_mut_ptr();
            copy_nonoverlapping(src_ptr, dst_ptr, size_of::<RxGlobalHeader>());
            header.msg_id
        };

        let mut cursor = size_of::<RxGlobalHeader>();
        unsafe {
            const FIXED_NUM_UNIT: Float = ULTRASOUND_WAVELENGTH / 256.0;
            for device in 0..num_devices {
                let mut foci = Vec::with_capacity(send_size as usize * 10);
                for i in 0..(send_size as usize) {
                    let v64 = geometry.local_position(device, seq.control_points()[seq.sent() + i]);
                    let x = (v64.x / FIXED_NUM_UNIT) as i32 as u32;
                    let y = (v64.y / FIXED_NUM_UNIT) as i32 as u32;
                    let z = (v64.z / FIXED_NUM_UNIT) as i32 as u32;
                    foci.push((x & 0x000000FF) as u8);
                    foci.push(((x & 0x0000FF00) >> 8) as u8);
                    foci.push((((x & 0x80000000) >> 24) | ((x & 0x007F0000) >> 16)) as u8);
                    foci.push((y & 0x000000FF) as u8);
                    foci.push(((y & 0x0000FF00) >> 8) as u8);
                    foci.push((((y & 0x80000000) >> 24) | ((y & 0x007F0000) >> 16)) as u8);
                    foci.push((z & 0x000000FF) as u8);
                    foci.push(((z & 0x0000FF00) >> 8) as u8);
                    foci.push((((z & 0x80000000) >> 24) | ((z & 0x007F0000) >> 16)) as u8);
                    foci.push(0xFF); // amp
                }
                let src_ptr = foci.as_ptr() as *const u8;
                let dst_ptr = body.as_mut_ptr().add(cursor);

                copy_nonoverlapping(src_ptr, dst_ptr, foci.len());
                cursor += NUM_TRANS_IN_UNIT * 2;
            }
        }
        seq.send(send_size);
        (msg_id, body)
    }

    unsafe fn convert_to_u8_slice<T: Sized>(p: &T) -> &[u8] {
        ::std::slice::from_raw_parts((p as *const T) as *const u8, ::std::mem::size_of::<T>())
    }
}
