/*
 * File: controller.rs
 * Project: src
 * Created Date: 27/04/2022
 * Author: Shun Suzuki
 * -----
 * Last Modified: 09/05/2022
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2022 Hapis Lab. All rights reserved.
 *
 */

use std::{
    marker::PhantomData,
    sync::atomic::{self, AtomicU8},
};

use anyhow::Result;
use itertools::Itertools;

use autd3_core::{
    geometry::{Geometry, Transducer},
    interface::{DatagramBody, DatagramHeader, Empty, Filled, Sendable},
    is_msg_processed,
    link::Link,
    FirmwareInfo, RxDatagram, TxDatagram, EC_DEVICE_PER_FRAME, EC_TRAFFIC_DELAY,
    MSG_NORMAL_BEGINNING, MSG_NORMAL_END, NUM_TRANS_IN_UNIT,
};

use crate::{prelude::Null, SilencerConfig};

static MSG_ID: AtomicU8 = AtomicU8::new(MSG_NORMAL_BEGINNING);

pub struct Sender<'a, 'b, L: Link, T: Transducer, S: Sendable<T>, H, B> {
    cnt: &'a mut Controller<L, T>,
    buf: &'b mut S,
    sent: bool,
    _head: PhantomData<H>,
    _body: PhantomData<B>,
}

impl<'a, 'b, L: Link, T: Transducer, S: Sendable<T>> Sender<'a, 'b, L, T, S, Empty, Empty> {
    pub fn new(cnt: &'a mut Controller<L, T>, s: &'b mut S) -> Sender<'a, 'b, L, T, S, S::H, S::B> {
        Sender {
            cnt,
            buf: s,
            sent: false,
            _head: PhantomData,
            _body: PhantomData,
        }
    }
}

impl<'a, 'b, L: Link, T: Transducer, S: Sendable<T>> Sender<'a, 'b, L, T, S, Filled, Empty> {
    pub fn send<B: DatagramBody<T>>(mut self, b: &'b mut B) -> Result<bool> {
        b.init()?;
        self.buf.init()?;

        let mut succsess = true;
        loop {
            self.cnt.tx_buf.clear();

            autd3_core::force_fan(&mut self.cnt.tx_buf, self.cnt.force_fan);
            autd3_core::reads_fpga_info(&mut self.cnt.tx_buf, self.cnt.reads_fpga_info);

            let msg_id = self.cnt.get_id();
            self.buf
                .pack(msg_id, &self.cnt.geometry, &mut self.cnt.tx_buf)?;
            b.pack(msg_id, &self.cnt.geometry, &mut self.cnt.tx_buf)?;
            self.cnt.link.send(&self.cnt.tx_buf)?;
            succsess &= self.cnt.wait_msg_processed(50)?;
            if !succsess || (self.buf.is_finished() && b.is_finished()) {
                break;
            }
        }
        self.sent = true;
        Ok(succsess)
    }

    pub fn flush(mut self) -> Result<bool> {
        self.buf.init()?;

        let mut succsess = true;
        loop {
            self.cnt.tx_buf.clear();

            autd3_core::force_fan(&mut self.cnt.tx_buf, self.cnt.force_fan);
            autd3_core::reads_fpga_info(&mut self.cnt.tx_buf, self.cnt.reads_fpga_info);

            let msg_id = self.cnt.get_id();
            self.buf
                .pack(msg_id, &self.cnt.geometry, &mut self.cnt.tx_buf)?;
            self.cnt.link.send(&self.cnt.tx_buf)?;
            succsess &= self.cnt.wait_msg_processed(50)?;
            if !succsess || self.buf.is_finished() {
                break;
            }
        }
        self.sent = true;
        Ok(succsess)
    }
}

impl<'a, 'b, L: Link, T: Transducer, S: Sendable<T>> Sender<'a, 'b, L, T, S, Empty, Filled> {
    pub fn send<H: DatagramHeader>(mut self, b: &'b mut H) -> Result<bool> {
        b.init()?;
        self.buf.init()?;

        let mut succsess = true;
        loop {
            self.cnt.tx_buf.clear();

            autd3_core::force_fan(&mut self.cnt.tx_buf, self.cnt.force_fan);
            autd3_core::reads_fpga_info(&mut self.cnt.tx_buf, self.cnt.reads_fpga_info);

            let msg_id = self.cnt.get_id();
            b.pack(msg_id, &mut self.cnt.tx_buf)?;
            self.buf
                .pack(msg_id, &self.cnt.geometry, &mut self.cnt.tx_buf)?;
            self.cnt.link.send(&self.cnt.tx_buf)?;
            succsess &= self.cnt.wait_msg_processed(50)?;
            if !succsess || (self.buf.is_finished() && b.is_finished()) {
                break;
            }
        }
        self.sent = true;
        Ok(succsess)
    }

    pub fn flush(mut self) -> Result<bool> {
        let mut succsess = true;
        loop {
            self.cnt.tx_buf.clear();

            autd3_core::force_fan(&mut self.cnt.tx_buf, self.cnt.force_fan);
            autd3_core::reads_fpga_info(&mut self.cnt.tx_buf, self.cnt.reads_fpga_info);

            let msg_id = self.cnt.get_id();
            self.buf
                .pack(msg_id, &self.cnt.geometry, &mut self.cnt.tx_buf)?;
            self.cnt.link.send(&self.cnt.tx_buf)?;
            succsess &= self.cnt.wait_msg_processed(50)?;
            if !succsess || self.buf.is_finished() {
                break;
            }
        }
        self.sent = true;
        Ok(succsess)
    }
}

impl<'a, 'b, L: Link, T: Transducer, S: Sendable<T>, H, B> Drop for Sender<'a, 'b, L, T, S, H, B> {
    fn drop(&mut self) {
        if !self.sent {
            self.buf.init().unwrap();

            let mut succsess = true;
            loop {
                self.cnt.tx_buf.clear();

                autd3_core::force_fan(&mut self.cnt.tx_buf, self.cnt.force_fan);
                autd3_core::reads_fpga_info(&mut self.cnt.tx_buf, self.cnt.reads_fpga_info);

                let msg_id = self.cnt.get_id();
                self.buf
                    .pack(msg_id, &self.cnt.geometry, &mut self.cnt.tx_buf)
                    .unwrap();
                self.cnt.link.send(&self.cnt.tx_buf).unwrap();
                succsess &= self.cnt.wait_msg_processed(50).unwrap();
                if !succsess || self.buf.is_finished() {
                    break;
                }
            }
            self.sent = true;
        }
    }
}

pub struct Controller<L: Link, T: Transducer> {
    link: L,
    geometry: Geometry<T>,
    tx_buf: TxDatagram,
    rx_buf: RxDatagram,
    pub check_ack: bool,
    pub force_fan: bool,
    pub reads_fpga_info: bool,
}

impl<L: Link, T: Transducer> Controller<L, T> {
    pub fn open(geometry: Geometry<T>, link: L) -> Result<Controller<L, T>> {
        let mut link = link;
        link.open()?;
        let num_devices = geometry.num_devices();
        Ok(Controller {
            link,
            geometry,
            tx_buf: TxDatagram::new(num_devices),
            rx_buf: RxDatagram::new(num_devices),
            check_ack: false,
            force_fan: false,
            reads_fpga_info: false,
        })
    }
}

impl<L: Link, T: Transducer> Controller<L, T> {
    pub fn geometry(&self) -> &Geometry<T> {
        &self.geometry
    }

    /// Send header and body to the devices
    ///
    /// # Arguments
    ///
    /// * `header` - Header
    /// * `body` - Body
    ///
    pub fn send<'a, 'b, S: Sendable<T>>(
        &'a mut self,
        s: &'b mut S,
    ) -> Sender<'a, 'b, L, T, S, S::H, S::B> {
        Sender::new(self, s)
    }

    pub fn config_silencer(&mut self, config: SilencerConfig) -> Result<bool> {
        self.tx_buf.clear();

        autd3_core::force_fan(&mut self.tx_buf, self.force_fan);
        autd3_core::reads_fpga_info(&mut self.tx_buf, self.reads_fpga_info);

        let msg_id = self.get_id();
        autd3_core::config_silencer(msg_id, config.cycle, config.step, &mut self.tx_buf)?;

        self.link.send(&self.tx_buf)?;
        self.wait_msg_processed(50)
    }

    /// Clear all data
    pub fn clear(&mut self) -> Result<bool> {
        let check_ack = self.check_ack;
        self.check_ack = true;
        self.tx_buf.clear();
        autd3_core::clear(&mut self.tx_buf);
        self.link.send(&self.tx_buf)?;
        let success = self.wait_msg_processed(200)?;
        self.check_ack = check_ack;
        Ok(success)
    }

    pub fn synchronize(&mut self) -> Result<bool> {
        self.tx_buf.clear();

        autd3_core::force_fan(&mut self.tx_buf, self.force_fan);
        autd3_core::reads_fpga_info(&mut self.tx_buf, self.reads_fpga_info);

        let msg_id = self.get_id();
        let cycles: Vec<[u16; NUM_TRANS_IN_UNIT]> = self
            .geometry
            .transducers()
            .map(|tr| tr.cycle())
            .chunks(NUM_TRANS_IN_UNIT)
            .into_iter()
            .map(|c| c.collect::<Vec<_>>())
            .map(|v| v.try_into().unwrap())
            .collect();

        autd3_core::sync(msg_id, self.link.cycle_ticks(), &cycles, &mut self.tx_buf)?;

        self.link.send(&self.tx_buf)?;
        self.wait_msg_processed(50)
    }

    /// Stop outputting
    pub fn stop(&mut self) -> Result<bool>
    where
        Null<T>: DatagramBody<T>,
    {
        let config = SilencerConfig::default();
        let res = self.config_silencer(config)?;

        let mut g = Null::new();

        let res = res & self.send(&mut g).flush()?;

        Ok(res)
    }

    /// Close controller
    pub fn close(&mut self) -> Result<bool>
    where
        Null<T>: DatagramBody<T>,
    {
        let res = self.stop()?;
        let res = res & self.clear()?;
        self.link.close()?;
        Ok(res)
    }

    /// Return firmware information of the devices
    pub fn firmware_infos(&mut self) -> Result<Vec<FirmwareInfo>> {
        let check_ack = self.check_ack;
        self.check_ack = true;

        autd3_core::cpu_version(&mut self.tx_buf);
        self.link.send(&self.tx_buf)?;
        self.wait_msg_processed(50)?;
        let cpu_versions = self
            .rx_buf
            .messages()
            .iter()
            .map(|rx| rx.ack)
            .collect::<Vec<_>>();

        autd3_core::fpga_version(&mut self.tx_buf);
        self.link.send(&self.tx_buf)?;
        self.wait_msg_processed(50)?;
        let fpga_versions = self
            .rx_buf
            .messages()
            .iter()
            .map(|rx| rx.ack)
            .collect::<Vec<_>>();

        autd3_core::fpga_functions(&mut self.tx_buf);
        self.link.send(&self.tx_buf)?;
        self.wait_msg_processed(50)?;
        let fpga_functions = self
            .rx_buf
            .messages()
            .iter()
            .map(|rx| rx.ack)
            .collect::<Vec<_>>();

        self.check_ack = check_ack;

        Ok((0..self.geometry.num_devices())
            .map(|i| FirmwareInfo::new(0, cpu_versions[i], fpga_versions[i], fpga_functions[i]))
            .collect())
    }
}

impl<L: Link, T: Transducer> Controller<L, T> {
    pub fn get_id(&self) -> u8 {
        MSG_ID.fetch_add(1, atomic::Ordering::SeqCst);
        let _ = MSG_ID.compare_exchange(
            MSG_NORMAL_END,
            MSG_NORMAL_BEGINNING,
            atomic::Ordering::SeqCst,
            atomic::Ordering::SeqCst,
        );
        MSG_ID.load(atomic::Ordering::SeqCst)
    }

    fn wait_msg_processed(&mut self, max_trial: usize) -> Result<bool> {
        if !self.check_ack {
            return Ok(true);
        }
        let msg_id = self.tx_buf.header().msg_id;
        let wait = (EC_TRAFFIC_DELAY * 1000.0 / EC_DEVICE_PER_FRAME as f64
            * self.geometry.num_devices() as f64)
            .ceil() as u64;
        let mut success = false;
        for _ in 0..max_trial {
            if !self.link.receive(&mut self.rx_buf)? {
                continue;
            }
            if is_msg_processed(msg_id, &self.rx_buf) {
                success = true;
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(wait));
        }

        Ok(success)
    }
}
