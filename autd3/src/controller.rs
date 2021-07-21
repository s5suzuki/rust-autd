/*
 * File: Controller.rs
 * Project: src
 * Created Date: 25/05/2021
 * Author: Shun Suzuki
 * -----
 * Last Modified: 21/07/2021
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2021 Hapis Lab. All rights reserved.
 *
 */

use crate::{
    error::AutdError,
    gain::Null,
    stm_controller::{StmController, StmTimerCallback},
};
use anyhow::Result;
use autd3_core::{
    ec_config::{EC_DEVICE_PER_FRAME, EC_INPUT_FRAME_SIZE, EC_OUTPUT_FRAME_SIZE, EC_TRAFFIC_DELAY},
    firmware_version::FirmwareInfo,
    gain::Gain,
    geometry::Geometry,
    hardware_defined::{CommandType, RxGlobalControlFlags, RxGlobalHeader, NUM_TRANS_IN_UNIT},
    link::Link,
    logic::Logic,
    modulation::Modulation,
    sequence::{GainSequence, PointSequence, Sequence},
};
use std::thread;

pub(crate) struct ControllerProps {
    pub(crate) geometry: Geometry,
    pub(crate) silent_mode: bool,
    pub(crate) reads_fpga_info: bool,
    pub(crate) force_fan: bool,
    pub(crate) seq_mode: bool,
}

/// Controller for AUTD3
pub struct Controller<L: Link> {
    link: L,
    geometry: Geometry,
    /// Silent mode flag. Default is true. **The flags in the actual devices will be update after [update_ctrl_flags](#method.update_ctrl_flags) or [send](#method.send) functions is called.**
    pub silent_mode: bool,
    /// If true, the devices return FPGA information in all frames. Default is false. **The flags in the actual devices will be update after [update_ctrl_flags](#method.update_ctrl_flags) or [send](#method.send) functions is called.**
    pub reads_fpga_info: bool,
    /// If true, the fan will be forced to start. Default is false. **The flags in the actual devices will be update after [update_ctrl_flags](#method.update_ctrl_flags) or [send](#method.send) functions is called.**
    pub force_fan: bool,
    seq_mode: bool,
    tx_buf: Vec<u8>,
    rx_buf: Vec<u8>,
    fpga_infos: Vec<u8>,
    delay_offsets: Vec<[u16; NUM_TRANS_IN_UNIT]>,
}

impl<L: Link> Controller<L> {
    pub(crate) fn new(link: L, props: ControllerProps) -> Self {
        let num_devices = props.geometry.num_devices();
        Self {
            link,
            geometry: props.geometry,
            silent_mode: props.silent_mode,
            reads_fpga_info: props.reads_fpga_info,
            force_fan: props.force_fan,
            seq_mode: props.seq_mode,
            tx_buf: vec![0x00; num_devices * EC_OUTPUT_FRAME_SIZE],
            rx_buf: vec![0x00; num_devices * EC_INPUT_FRAME_SIZE],
            fpga_infos: vec![0x00; num_devices],
            delay_offsets: vec![[0xFF00; NUM_TRANS_IN_UNIT]; num_devices],
        }
    }

    /// Construct controller with `geometry` and open `link`.
    ///
    /// # Arguments
    ///
    /// * `geometry` - Geometry of the devices
    /// * `link` - Link to the device
    ///
    pub fn open(geometry: Geometry, link: L) -> Result<Self> {
        let mut link = link;
        link.open()?;
        Ok(Self::new(
            link,
            ControllerProps {
                geometry,
                reads_fpga_info: false,
                seq_mode: false,
                force_fan: false,
                silent_mode: true,
            },
        ))
    }

    pub fn geometry(&self) -> &Geometry {
        &self.geometry
    }

    /// Return FPGA information of the devices (the first bit represent whether the fan is running).
    ///
    /// To use this function, set `reads_fpga_info` true.
    ///
    pub fn fpga_infos(&mut self) -> Result<&[u8]> {
        self.link.read(&mut self.rx_buf)?;
        for i in 0..self.geometry.num_devices() {
            self.fpga_infos[i] = self.rx_buf[2 * i];
        }
        Ok(&self.fpga_infos)
    }

    /// Return the link is opened.
    pub fn is_open(&self) -> bool {
        self.link.is_open()
    }

    /// Update control flags
    pub async fn update_ctrl_flags(&mut self) -> Result<bool> {
        self.send_header(CommandType::Op).await
    }

    /// Set output delay
    ///
    /// # Arguments
    ///
    /// * `delay` -  delay for each transducer in units of ultrasound period (i.e. 25us).
    ///
    pub async fn set_output_delay(&mut self, delay: &[[u8; NUM_TRANS_IN_UNIT]]) -> Result<bool> {
        let num_devices = self.geometry().num_devices();
        if delay.len() != num_devices {
            return Err(AutdError::DelayOutOfRange(delay.len(), num_devices).into());
        }

        for (dev, d) in delay.iter().enumerate() {
            for (i, &v) in d.iter().enumerate() {
                self.delay_offsets[dev][i] = (self.delay_offsets[dev][i] & 0xFF00) | v as u16;
            }
        }
        self.send_delay_offset().await
    }

    /// Set duty offset
    ///
    /// # Arguments
    ///
    /// * `offset` - duty offset for each transducer (only the first bit is used)
    ///
    pub async fn set_duty_offset(&mut self, offset: &[[u8; NUM_TRANS_IN_UNIT]]) -> Result<bool> {
        let num_devices = self.geometry().num_devices();
        if offset.len() != num_devices {
            return Err(AutdError::DelayOutOfRange(offset.len(), num_devices).into());
        }
        for (dev, e) in offset.iter().enumerate() {
            for (i, &v) in e.iter().enumerate() {
                self.delay_offsets[dev][i] =
                    (self.delay_offsets[dev][i] & 0x00FF) | (((v as u16) << 8) & 0xFF00);
            }
        }
        self.send_delay_offset().await
    }

    /// Set output delay and offset
    ///
    /// # Arguments
    ///
    /// * `delay_offset` - lower 8bits is delay and 8-th bit is offset
    ///
    pub async fn set_delay_offset(
        &mut self,
        delay_offset: &[[u16; NUM_TRANS_IN_UNIT]],
    ) -> Result<bool> {
        let num_devices = self.geometry().num_devices();
        if delay_offset.len() != num_devices {
            return Err(AutdError::DelayOutOfRange(delay_offset.len(), num_devices).into());
        }
        for (dev, d) in delay_offset.iter().enumerate() {
            unsafe {
                std::ptr::copy_nonoverlapping(
                    d.as_ptr(),
                    self.delay_offsets[dev].as_mut_ptr(),
                    NUM_TRANS_IN_UNIT,
                );
            }
        }
        self.send_delay_offset().await
    }

    /// Clear all data
    pub async fn clear(&mut self) -> Result<bool> {
        self.send_header(CommandType::Clear).await
    }

    /// Close controller
    pub async fn close(&mut self) -> Result<()> {
        self.stop().await?;
        self.link.close()
    }

    /// Stop outputting
    pub async fn stop(&mut self) -> Result<bool> {
        let mut null = Null::new();
        self.send_gain(&mut null).await?;
        self.send_header(CommandType::Pause).await
    }

    /// Pause outputting
    pub async fn pause(&mut self) -> Result<bool> {
        self.send_header(CommandType::Pause).await
    }

    /// Resume outputting
    pub async fn resume(&mut self) -> Result<bool> {
        self.send_header(CommandType::Resume).await
    }

    /// Send gain and modulation to the devices
    ///
    /// # Arguments
    ///
    /// * `g` - Gain
    /// * `m` - Modulation
    ///
    pub async fn send<G: Gain, M: Modulation>(&mut self, g: &mut G, m: &mut M) -> Result<bool> {
        m.build()?;

        self.seq_mode = false;
        g.build(&self.geometry)?;

        let mut size = 0;
        Logic::pack_body(g, &mut self.tx_buf, &mut size);

        loop {
            let mut msg_id = 0;
            Logic::pack_header_mod(m, self.ctrl_flag(), &mut self.tx_buf, &mut msg_id);

            self.link.send(&self.tx_buf[0..size])?;
            let r = self.wait_msg_processed(msg_id, 50).await?;
            if !r || m.finished() {
                return Ok(r);
            }
        }
    }

    /// Send gain to the devices
    ///
    /// # Arguments
    ///
    /// * `g` - Gain
    ///
    pub async fn send_gain<G: Gain>(&mut self, g: &mut G) -> Result<bool> {
        self.seq_mode = false;
        g.build(&self.geometry)?;

        let mut size = 0;
        Logic::pack_body(g, &mut self.tx_buf, &mut size);

        let mut msg_id = 0;
        Logic::pack_header(
            CommandType::Op,
            self.ctrl_flag(),
            &mut self.tx_buf,
            &mut msg_id,
        );

        self.link.send(&self.tx_buf[0..size])?;
        self.wait_msg_processed(msg_id, 50).await
    }

    /// Send modulation to the devices
    ///
    /// # Arguments
    ///
    /// * `m` - Modulation
    ///
    pub async fn send_modulation<M: Modulation>(&mut self, m: &mut M) -> Result<bool> {
        m.build()?;

        let size = std::mem::size_of::<RxGlobalHeader>();
        loop {
            let mut msg_id = 0;
            Logic::pack_header_mod(m, self.ctrl_flag(), &mut self.tx_buf, &mut msg_id);
            self.link.send(&self.tx_buf[0..size])?;
            let r = self.wait_msg_processed(msg_id, 50).await?;
            if !r || m.finished() {
                return Ok(r);
            }
        }
    }

    /// Send sequence to the devices
    ///
    /// # Arguments
    ///
    /// * `s` - Sequence
    ///
    pub async fn send_seq(&mut self, s: &mut PointSequence) -> Result<bool> {
        self.seq_mode = true;
        loop {
            let mut msg_id = 0;
            Logic::pack_header(
                CommandType::PointSeqMode,
                self.ctrl_flag(),
                &mut self.tx_buf,
                &mut msg_id,
            );
            let mut size = 0;
            Logic::pack_seq(s, &self.geometry, &mut self.tx_buf, &mut size);
            self.link.send(&self.tx_buf[0..size])?;

            let r = self.wait_msg_processed(msg_id, 50).await?;
            if !r || s.finished() {
                return Ok(r);
            }
        }
    }

    /// Send sequence to the devices
    ///
    /// # Arguments
    ///
    /// * `s` - Sequence
    ///
    pub async fn send_gain_seq(&mut self, s: &mut GainSequence) -> Result<bool> {
        self.seq_mode = true;
        loop {
            let mut msg_id = 0;
            Logic::pack_header(
                CommandType::GainSeqMode,
                self.ctrl_flag(),
                &mut self.tx_buf,
                &mut msg_id,
            );
            let mut size = 0;
            Logic::pack_gain_seq(s, &self.geometry, &mut self.tx_buf, &mut size);
            self.link.send(&self.tx_buf[0..size])?;
            let r = self.wait_msg_processed(msg_id, 50).await?;
            if !r || s.finished() {
                return Ok(r);
            }
        }
    }

    /// Return firmware information of the devices
    pub async fn firmware_infos(&mut self) -> Result<Vec<FirmwareInfo>> {
        fn concat_byte(high: u8, low: u16) -> u16 {
            (((high as u16) << 8) & 0xFF00) | (low & 0x00FF)
        }

        let num_devices = self.geometry.num_devices();
        let mut cpu_versions = vec![0x0000; num_devices];
        self.send_header(CommandType::ReadCpuVerLsb).await?;
        for (i, ver) in cpu_versions.iter_mut().enumerate().take(num_devices) {
            *ver = self.rx_buf[2 * i] as u16;
        }
        self.send_header(CommandType::ReadCpuVerMsb).await?;
        for (i, ver) in cpu_versions.iter_mut().enumerate().take(num_devices) {
            *ver = concat_byte(self.rx_buf[2 * i], *ver);
        }

        let mut fpga_versions = vec![0x0000; num_devices];
        self.send_header(CommandType::ReadFpgaVerLsb).await?;
        for (i, ver) in fpga_versions.iter_mut().enumerate().take(num_devices) {
            *ver = self.rx_buf[2 * i] as u16;
        }
        self.send_header(CommandType::ReadFpgaVerMsb).await?;
        for (i, ver) in fpga_versions.iter_mut().enumerate().take(num_devices) {
            *ver = concat_byte(self.rx_buf[2 * i], *ver);
        }

        let mut infos = Vec::with_capacity(num_devices);
        for i in 0..num_devices {
            infos.push(FirmwareInfo::new(
                i as u16,
                cpu_versions[i],
                fpga_versions[i],
            ));
        }
        Ok(infos)
    }

    /// Return Spatio-temporal controller
    pub fn stm(self) -> StmController<L> {
        StmController {
            callback: StmTimerCallback::new(self.link),
            props: ControllerProps {
                geometry: self.geometry,
                reads_fpga_info: self.reads_fpga_info,
                seq_mode: self.seq_mode,
                silent_mode: self.silent_mode,
                force_fan: self.force_fan,
            },
        }
    }

    fn ctrl_flag(&self) -> RxGlobalControlFlags {
        let mut header = RxGlobalControlFlags::NONE;
        if self.silent_mode {
            header |= RxGlobalControlFlags::SILENT;
        }
        if self.seq_mode {
            header |= RxGlobalControlFlags::SEQ_MODE;
        }
        if self.reads_fpga_info {
            header |= RxGlobalControlFlags::READ_FPGA_INFO;
        }
        if self.force_fan {
            header |= RxGlobalControlFlags::FORCE_FAN;
        }
        header
    }

    async fn send_header(&mut self, cmd: CommandType) -> Result<bool> {
        let send_size = std::mem::size_of::<RxGlobalHeader>();
        let mut msg_id: u8 = 0;
        Logic::pack_header(cmd, self.ctrl_flag(), &mut self.tx_buf, &mut msg_id);
        self.link.send(&self.tx_buf[0..send_size])?;
        self.wait_msg_processed(msg_id, 50).await
    }

    async fn send_delay_offset(&mut self) -> Result<bool> {
        let mut msg_id: u8 = 0;
        Logic::pack_header(
            CommandType::SetDelay,
            self.ctrl_flag(),
            &mut self.tx_buf,
            &mut msg_id,
        );
        let mut size = 0;
        let num_devices = self.geometry().num_devices();
        Logic::pack_delay_offset(
            &self.delay_offsets,
            num_devices,
            &mut self.tx_buf,
            &mut size,
        );
        self.link.send(&self.tx_buf[0..size])?;
        self.wait_msg_processed(msg_id, 50).await
    }

    async fn wait_msg_processed(&mut self, msg_id: u8, max_trial: usize) -> Result<bool> {
        let num_devices = self.geometry.num_devices();
        let wait = (EC_TRAFFIC_DELAY * 1000.0 / EC_DEVICE_PER_FRAME as f64 * num_devices as f64)
            .ceil() as u64;
        for _ in 0..max_trial {
            if !self.link.read(&mut self.rx_buf)? {
                continue;
            }
            if Logic::is_msg_processed(num_devices, msg_id, &self.rx_buf) {
                return Ok(true);
            }
            thread::sleep(std::time::Duration::from_millis(wait));
        }
        Ok(false)
    }
}
