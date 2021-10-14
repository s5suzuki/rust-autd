/*
 * File: emulator_link.rs
 * Project: src
 * Created Date: 27/05/2021
 * Author: Shun Suzuki
 * -----
 * Last Modified: 14/10/2021
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2021 Hapis Lab. All rights reserved.
 *
 */

use std::{mem::size_of, net::UdpSocket};

use anyhow::Result;

use autd3_core::{
    geometry::Geometry,
    hardware_defined::CommandType,
    hardware_defined::{FPGAControlFlags, GlobalHeader, NUM_TRANS_IN_UNIT},
    link::Link,
};

pub struct EmulatorLink {
    port: u16,
    socket: Option<UdpSocket>,
    geometry_buf: Vec<u8>,
    last_msg_id: u8,
    last_cmd: CommandType,
}

impl EmulatorLink {
    pub fn new(port: u16, geometry: &Geometry) -> Self {
        let vec_size = 9 * size_of::<f32>();
        let size = size_of::<GlobalHeader>() + geometry.num_devices() * vec_size;
        let mut geometry_buf = vec![0; size];

        unsafe {
            let uh = geometry_buf.as_mut_ptr() as *mut GlobalHeader;
            (*uh).msg_id = 0x00;
            (*uh).ctrl_flag = FPGAControlFlags::NONE;
            (*uh).command = CommandType::EmulatorSetGeometry;
            (*uh).mod_size = 0x00;

            let mut cursor = geometry_buf.as_mut_ptr().add(size_of::<GlobalHeader>()) as *mut f32;
            for i in 0..geometry.num_devices() {
                let trans_id = i * NUM_TRANS_IN_UNIT;
                let origin = geometry.position_by_global_idx(trans_id);
                let right = geometry.x_direction(i);
                let up = geometry.y_direction(i);

                cursor.write(origin.x as f32);
                cursor.add(1).write(origin.y as f32);
                cursor.add(2).write(origin.z as f32);
                cursor.add(3).write(right.x as f32);
                cursor.add(4).write(right.y as f32);
                cursor.add(5).write(right.z as f32);
                cursor.add(6).write(up.x as f32);
                cursor.add(7).write(up.y as f32);
                cursor.add(8).write(up.z as f32);
                cursor = cursor.add(9);
            }
        }

        Self {
            port,
            geometry_buf,
            socket: None,
            last_msg_id: 0,
            last_cmd: CommandType::Op,
        }
    }
}

impl Link for EmulatorLink {
    fn open(&mut self) -> Result<()> {
        let socket = UdpSocket::bind("0.0.0.0:8080")?;
        let remote_addr = format!("127.0.0.1:{}", self.port);
        socket.connect(remote_addr)?;
        socket.send(&self.geometry_buf)?;
        self.socket = Some(socket);
        Ok(())
    }

    fn close(&mut self) -> Result<()> {
        self.socket = None;
        Ok(())
    }

    fn send(&mut self, data: &[u8]) -> Result<bool> {
        if let Some(socket) = &self.socket {
            socket.try_clone()?.send(data)?;
            unsafe {
                let uh = data.as_ptr() as *mut GlobalHeader;
                self.last_msg_id = (*uh).msg_id;
                self.last_cmd = (*uh).command;
            }
        }
        Ok(true)
    }

    fn read(&mut self, data: &mut [u8]) -> Result<bool> {
        for i in 0..(data.len() / 2) {
            data[i * 2 + 1] = self.last_msg_id;
        }

        let mut set = |value: u8| {
            for i in 0..(data.len() / 2) {
                data[i * 2] = value;
            }
        };

        match self.last_cmd {
            CommandType::Op => (),
            CommandType::ReadCpuVerLsb => set(0xFF),
            CommandType::ReadCpuVerMsb => set(0xFF),
            CommandType::ReadFpgaVerLsb => set(0xFF),
            CommandType::ReadFpgaVerMsb => set(0xFF),
            CommandType::PointSeqMode => (),
            CommandType::GainSeqMode => (),
            CommandType::Clear => (),
            CommandType::SetDelay => (),
            CommandType::EmulatorSetGeometry => (),
        }

        Ok(true)
    }

    fn is_open(&self) -> bool {
        self.socket.is_some()
    }
}
