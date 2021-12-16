/*
 * File: emulator_link.rs
 * Project: src
 * Created Date: 27/05/2021
 * Author: Shun Suzuki
 * -----
 * Last Modified: 16/12/2021
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2021 Hapis Lab. All rights reserved.
 *
 */

use std::{mem::size_of, net::UdpSocket};

use anyhow::Result;

use autd3_core::{
    geometry::Geometry,
    hardware_defined::{
        CPUControlFlags, FPGAControlFlags, RxDatagram, TxDatagram, MSG_EMU_GEOMETRY_SET,
        MSG_RD_CPU_V_LSB, MSG_RD_CPU_V_MSB, MSG_RD_FPGA_V_LSB, MSG_RD_FPGA_V_MSB,
        NUM_TRANS_IN_UNIT,
    },
    link::Link,
};

pub struct EmulatorLink {
    port: u16,
    socket: Option<UdpSocket>,
    geometry_buf: TxDatagram,
    last_msg_id: u8,
}

impl EmulatorLink {
    pub fn new(port: u16, geometry: &Geometry) -> Self {
        let mut geometry_buf = TxDatagram::new(geometry.num_devices());
        let header = geometry_buf.header_mut();
        header.msg_id = MSG_EMU_GEOMETRY_SET;
        header.cpu_flag = CPUControlFlags::NONE;
        header.fpga_flag = FPGAControlFlags::NONE;
        header.mod_size = 0x00;

        for (device, buf) in geometry.devices().iter().zip(
            geometry_buf
                .body_data_mut::<[f32; NUM_TRANS_IN_UNIT * size_of::<u16>() / size_of::<f64>()]>(),
        ) {
            let origin = device.transducers()[0].position();
            let right = device.transducers()[0].x_direction();
            let up = device.transducers()[0].y_direction();
            buf[0] = origin.x as _;
            buf[1] = origin.y as _;
            buf[2] = origin.z as _;
            buf[3] = right.x as _;
            buf[4] = right.y as _;
            buf[5] = right.z as _;
            buf[6] = up.x as _;
            buf[7] = up.y as _;
            buf[8] = up.z as _;
        }

        Self {
            port,
            geometry_buf,
            socket: None,
            last_msg_id: 0,
        }
    }
}

impl Link for EmulatorLink {
    fn open(&mut self) -> Result<()> {
        let socket = UdpSocket::bind("0.0.0.0:8080")?;
        let remote_addr = format!("127.0.0.1:{}", self.port);
        socket.connect(remote_addr)?;
        socket.send(self.geometry_buf.data())?;
        self.socket = Some(socket);
        Ok(())
    }

    fn close(&mut self) -> Result<()> {
        self.socket = None;
        Ok(())
    }

    fn send(&mut self, tx: &TxDatagram) -> Result<bool> {
        if let Some(socket) = &self.socket {
            socket.try_clone()?.send(tx.data())?;
            self.last_msg_id = tx.header().msg_id;
        }
        Ok(true)
    }

    fn receive(&mut self, rx: &mut RxDatagram) -> Result<bool> {
        for r in rx.messages_mut() {
            r.msg_id = self.last_msg_id;
        }

        let mut set = |value: u8| {
            for r in rx.messages_mut() {
                r.ack = value;
            }
        };

        match self.last_msg_id {
            MSG_RD_CPU_V_LSB | MSG_RD_CPU_V_MSB | MSG_RD_FPGA_V_LSB | MSG_RD_FPGA_V_MSB => {
                set(0xFF)
            }
            _ => (),
        }

        Ok(true)
    }

    fn is_open(&self) -> bool {
        self.socket.is_some()
    }
}
