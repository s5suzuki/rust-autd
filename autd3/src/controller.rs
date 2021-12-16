/*
 * File: Controller.rs
 * Project: src
 * Created Date: 25/05/2021
 * Author: Shun Suzuki
 * -----
 * Last Modified: 16/12/2021
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2021 Hapis Lab. All rights reserved.
 *
 */

use std::sync::atomic::{self, AtomicU8};

use crate::{
    gain::Null,
    stm_controller::{StmController, StmTimerCallback},
};
use anyhow::Result;
use autd3_core::{
    datagrams::{CommonHeader, NullBody, SpecialMessageIdHeader},
    ec_config::{EC_DEVICE_PER_FRAME, EC_TRAFFIC_DELAY},
    firmware_version::FirmwareInfo,
    geometry::Geometry,
    hardware_defined::{
        is_msg_processed, CPUControlFlags, FPGAControlFlags, FPGAInfo, RxDatagram, TxDatagram,
        MSG_CLEAR, MSG_NORMAL_BASE,
    },
    interface::{IDatagramBody, IDatagramHeader},
    link::Link,
};

static MSG_ID: AtomicU8 = AtomicU8::new(MSG_NORMAL_BASE);

pub(crate) struct ControllerProps {
    pub(crate) geometry: Geometry,
    pub(crate) output_enable: bool,
    pub(crate) output_balance: bool,
    pub(crate) silent_mode: bool,
    pub(crate) reads_fpga_info: bool,
    pub(crate) force_fan: bool,
    pub(crate) check_ack: bool,
}

/// Controller for AUTD3
pub struct Controller<L: Link> {
    link: L,
    geometry: Geometry,
    /// Silent mode flag. Default is true. **The flags in the actual devices will be update after [update_fpga_flags](#method.update_fpga_flags) or [send](#method.send) functions is called.**
    pub silent_mode: bool,
    /// If true, the devices return FPGA information in all frames. Default is false. **The flags in the actual devices will be update after [update_fpga_flags](#method.update_fpga_flags) or [send](#method.send) functions is called.**
    pub reads_fpga_info: bool,
    /// If true, the fan will be forced to start. Default is false. **The flags in the actual devices will be update after [update_fpga_flags](#method.update_fpga_flags) or [send](#method.send) functions is called.**
    pub force_fan: bool,
    ///  If true, the applied voltage to transducers is dropped to GND while transducers are not being outputting. Default is false. **The flags in the actual devices will be update after [update_fpga_flags](#method.update_fpga_flags) or [send](#method.send) functions is called.**
    pub output_balance: bool,
    /// If true, this controller check ack from devices. Default is false.
    pub check_ack: bool,
    output_enable: bool,
    tx_buf: Option<TxDatagram>,
    rx_buf: Option<RxDatagram>,
    fpga_infos: Vec<FPGAInfo>,
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
            output_balance: props.output_balance,
            output_enable: props.output_enable,
            check_ack: props.check_ack,
            tx_buf: None,
            rx_buf: None,
            fpga_infos: vec![FPGAInfo::new(); num_devices],
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
                force_fan: false,
                silent_mode: true,
                output_balance: false,
                output_enable: false,
                check_ack: false,
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
    pub fn fpga_infos(&mut self) -> Result<&[FPGAInfo]> {
        let mut rx_buf = self.prepare_rx_buf();
        self.link.receive(&mut rx_buf)?;
        for (info, rx) in self.fpga_infos.iter_mut().zip(rx_buf.messages()) {
            info.copy_from(rx);
        }
        self.rx_buf = Some(rx_buf);
        Ok(&self.fpga_infos)
    }

    /// Return the link is opened.
    pub fn is_open(&self) -> bool {
        self.link.is_open()
    }

    /// Update control flags
    pub async fn update_fpga_flags(&mut self) -> Result<bool> {
        let mut header = CommonHeader::new(
            FPGAControlFlags::OUTPUT_ENABLE
                | FPGAControlFlags::OUTPUT_BALANCE
                | FPGAControlFlags::SILENT
                | FPGAControlFlags::FORCE_FAN,
        );
        self.send_header(&mut header).await
    }

    /// Clear all data
    pub async fn clear(&mut self) -> Result<bool> {
        let mut header = SpecialMessageIdHeader::new(MSG_CLEAR, FPGAControlFlags::all());
        self.send_header(&mut header).await
    }

    /// Close controller
    pub async fn close(&mut self) -> Result<()> {
        self.stop().await?;
        self.link.close()
    }

    /// Stop outputting
    pub async fn stop(&mut self) -> Result<bool> {
        let silent = self.silent_mode;
        self.silent_mode = true;
        let mut null = Null::new();
        self.send_body(&mut null).await?;
        self.silent_mode = silent;
        self.pause().await
    }

    /// Pause outputting
    pub async fn pause(&mut self) -> Result<bool> {
        self.output_enable = false;
        self.update_fpga_flags().await
    }

    /// Resume outputting
    pub async fn resume(&mut self) -> Result<bool> {
        self.output_enable = true;
        self.update_fpga_flags().await
    }

    /// Send header and body to the devices
    ///
    /// # Arguments
    ///
    /// * `header` - Header
    /// * `body` - Body
    ///
    pub async fn send<B: IDatagramBody, H: IDatagramHeader>(
        &mut self,
        header: &mut H,
        body: &mut B,
    ) -> Result<bool> {
        header.init()?;
        body.init();

        let fpga_flag = self.fpga_flag();
        let cpu_flag = self.cpu_flag();
        let mut tx_buf = self.prepare_tx_buf();
        let mut succsess = true;
        loop {
            let msg_id = Self::get_id();
            header.pack(msg_id, &mut tx_buf, fpga_flag, cpu_flag);
            body.pack(self.geometry(), &mut tx_buf)?;
            self.link.send(&tx_buf)?;
            succsess &= self.wait_msg_processed(msg_id, 50).await?;
            if !succsess || (header.is_finished() && body.is_finished()) {
                break;
            }
        }
        self.tx_buf = Some(tx_buf);
        Ok(succsess)
    }

    /// Send header to the devices
    ///
    /// # Arguments
    ///
    /// * `header` - Header
    ///
    pub async fn send_header<H: IDatagramHeader>(&mut self, header: &mut H) -> Result<bool> {
        let mut body = NullBody::new();
        self.send(header, &mut body).await
    }

    /// Send body to the devices
    ///
    /// # Arguments
    ///
    /// * `body` - Body
    ///
    pub async fn send_body<B: IDatagramBody>(&mut self, body: &mut B) -> Result<bool> {
        let mut header = CommonHeader::new(
            FPGAControlFlags::OUTPUT_ENABLE
                | FPGAControlFlags::OUTPUT_BALANCE
                | FPGAControlFlags::SILENT
                | FPGAControlFlags::FORCE_FAN,
        );
        self.send(&mut header, body).await
    }

    /// Return firmware information of the devices
    pub async fn firmware_infos(&mut self) -> Result<Vec<FirmwareInfo>> {
        fn concat_byte(high: u8, low: u16) -> u16 {
            (((high as u16) << 8) & 0xFF00) | (low & 0x00FF)
        }

        // For backward compatibility before 1.9
        const READ_CPU_VER_LSB: u8 = 0x02;
        const READ_CPU_VER_MSB: u8 = 0x03;
        const READ_FPGA_VER_LSB: u8 = 0x04;
        const READ_FPGA_VER_MSB: u8 = 0x05;
        async fn send_command<L: Link>(
            cnt: &mut Controller<L>,
            msg_id: u8,
            cmd: u8,
        ) -> Result<bool> {
            let mut special_message_id_header = SpecialMessageIdHeader::new(
                msg_id,
                FPGAControlFlags::OUTPUT_ENABLE
                    | FPGAControlFlags::OUTPUT_BALANCE
                    | FPGAControlFlags::SILENT
                    | FPGAControlFlags::FORCE_FAN,
            );
            let mut body = NullBody::new();

            special_message_id_header.init()?;
            body.init();

            let mut tx_buf = cnt.prepare_tx_buf();
            special_message_id_header.pack(0x00, &mut tx_buf, cnt.fpga_flag(), cnt.cpu_flag());
            body.pack(cnt.geometry(), &mut tx_buf)?;
            tx_buf.data_mut()[2] = cmd;
            cnt.link.send(&tx_buf)?;
            cnt.tx_buf = Some(tx_buf);
            cnt.wait_msg_processed(msg_id, 50).await
        }

        let check_ack = self.check_ack;
        self.check_ack = true;

        let num_devices = self.geometry.num_devices();
        let mut cpu_versions = vec![0x0000; num_devices];
        send_command(
            self,
            autd3_core::hardware_defined::MSG_RD_CPU_V_LSB,
            READ_CPU_VER_LSB,
        )
        .await?;
        let rx_buf = self.rx_buf.take().unwrap();
        for (ver, rx) in cpu_versions.iter_mut().zip(rx_buf.messages()) {
            *ver = rx.ack as u16;
        }
        self.rx_buf = Some(rx_buf);
        send_command(
            self,
            autd3_core::hardware_defined::MSG_RD_CPU_V_MSB,
            READ_CPU_VER_MSB,
        )
        .await?;
        let rx_buf = self.rx_buf.take().unwrap();
        for (ver, rx) in cpu_versions.iter_mut().zip(rx_buf.messages()) {
            *ver = concat_byte(rx.ack, *ver);
        }
        self.rx_buf = Some(rx_buf);

        let mut fpga_versions = vec![0x0000; num_devices];
        send_command(
            self,
            autd3_core::hardware_defined::MSG_RD_FPGA_V_LSB,
            READ_FPGA_VER_LSB,
        )
        .await?;
        let rx_buf = self.rx_buf.take().unwrap();
        for (ver, rx) in fpga_versions.iter_mut().zip(rx_buf.messages()) {
            *ver = rx.ack as u16;
        }
        self.rx_buf = Some(rx_buf);
        send_command(
            self,
            autd3_core::hardware_defined::MSG_RD_FPGA_V_MSB,
            READ_FPGA_VER_MSB,
        )
        .await?;
        let rx_buf = self.rx_buf.take().unwrap();
        for (ver, rx) in fpga_versions.iter_mut().zip(rx_buf.messages()) {
            *ver = concat_byte(rx.ack, *ver);
        }
        self.rx_buf = Some(rx_buf);

        self.check_ack = check_ack;

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
                silent_mode: self.silent_mode,
                force_fan: self.force_fan,
                output_balance: self.output_balance,
                output_enable: self.output_enable,
                check_ack: self.check_ack,
            },
        }
    }

    fn fpga_flag(&self) -> FPGAControlFlags {
        let mut header = FPGAControlFlags::NONE;
        if self.output_enable {
            header |= FPGAControlFlags::OUTPUT_ENABLE;
        }
        if self.output_balance {
            header |= FPGAControlFlags::OUTPUT_BALANCE;
        }
        if self.silent_mode {
            header |= FPGAControlFlags::SILENT;
        }
        if self.force_fan {
            header |= FPGAControlFlags::FORCE_FAN;
        }
        header
    }

    fn cpu_flag(&self) -> CPUControlFlags {
        let mut header = CPUControlFlags::NONE;
        if self.reads_fpga_info {
            header |= CPUControlFlags::READS_FPGA_INFO;
        }
        header
    }

    async fn wait_msg_processed(&mut self, msg_id: u8, max_trial: usize) -> Result<bool> {
        if !self.check_ack {
            return Ok(true);
        }
        let num_devices = self.geometry.num_devices();
        let wait = (EC_TRAFFIC_DELAY * 1000.0 / EC_DEVICE_PER_FRAME as f64 * num_devices as f64)
            .ceil() as u64;
        let mut success = false;
        let mut rx_buf = self.prepare_rx_buf();
        for _ in 0..max_trial {
            if !self.link.receive(&mut rx_buf)? {
                continue;
            }
            if is_msg_processed(msg_id, &rx_buf) {
                success = true;
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(wait)).await;
        }
        self.rx_buf = Some(rx_buf);
        Ok(success)
    }

    fn prepare_tx_buf(&mut self) -> TxDatagram {
        if self.tx_buf.is_none() {
            return TxDatagram::new(self.geometry().num_devices());
        }
        self.tx_buf.take().unwrap()
    }

    fn prepare_rx_buf(&mut self) -> RxDatagram {
        if self.rx_buf.is_none() {
            return RxDatagram::new(self.geometry().num_devices());
        }
        self.rx_buf.take().unwrap()
    }

    pub(crate) fn get_id() -> u8 {
        MSG_ID.fetch_add(1, atomic::Ordering::SeqCst);
        let _ = MSG_ID.compare_exchange(
            0xFF,
            MSG_NORMAL_BASE,
            atomic::Ordering::SeqCst,
            atomic::Ordering::SeqCst,
        );
        MSG_ID.load(atomic::Ordering::SeqCst)
    }
}
