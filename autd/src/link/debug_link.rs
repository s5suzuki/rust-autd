/*
 * File: debug_link.rs
 * Project: link
 * Created Date: 31/12/2020
 * Author: Shun Suzuki
 * -----
 * Last Modified: 31/12/2020
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2020 Hapis Lab. All rights reserved.
 *
 */

use std::{error::Error, io::Write};

use super::Link;

pub struct DebugLink<W: Write> {
    writer: W,
    last_msg_id: u8,
    is_open: bool,
}

impl<W: Write> DebugLink<W> {
    pub fn new(writer: W) -> Self {
        Self {
            writer,
            last_msg_id: 0,
            is_open: false,
        }
    }
}

impl<W: Write + Send> Link for DebugLink<W> {
    fn open(&mut self) -> Result<(), Box<dyn Error>> {
        self.writer.write_all(b"Call open()\n")?;
        self.is_open = true;
        Ok(())
    }

    fn close(&mut self) -> Result<(), Box<dyn Error>> {
        self.writer.write_all(b"Call close()\n")?;
        self.is_open = false;
        Ok(())
    }

    fn send(&mut self, data: &[u8]) -> Result<(), Box<dyn Error>> {
        self.last_msg_id = data[0];
        self.writer.write_all(b"Header:\n")?;
        self.writer
            .write_fmt(format_args!("\t msg_id: {:x}\n", self.last_msg_id))?;
        Ok(())
    }

    fn read(&mut self, data: &mut [u8], _buffer_len: usize) -> Result<(), Box<dyn Error>> {
        for i in data.iter_mut() {
            *i = self.last_msg_id;
        }

        Ok(())
    }

    fn is_open(&self) -> bool {
        self.is_open
    }
}
