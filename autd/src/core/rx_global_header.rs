/*
 * File: rx_global_header.rs
 * Project: src
 * Created Date: 21/11/2019
 * Author: Shun Suzuki
 * -----
 * Last Modified: 30/12/2020
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2019 Hapis Lab. All rights reserved.
 *
 */

use std::fmt;
use std::sync::atomic::{self, AtomicU8};

use super::consts::*;

const OP_MODE_MSG_ID_MIN: u8 = 0x20;
const OP_MODE_MSG_ID_MAX: u8 = 0xBF;

static MSG_ID: AtomicU8 = AtomicU8::new(OP_MODE_MSG_ID_MIN);

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum CommandType {
    CmdOp = 0x00,
    _CmdBramWrite = 0x01,
    CmdReadCpuVerLsb = 0x02,
    CmdReadCpuVerMsb = 0x03,
    CmdReadFpgaVerLsb = 0x04,
    CmdReadFpgaVerMsb = 0x05,
    CmdSeqMode = 0x06,
    CmdInitRefClock = 0x07,
    CmdCalibSeqClock = 0x08,
    CmdClear = 0x09,
    SetDelay = 0x0A,
}

#[repr(C)]
pub struct RxGlobalHeader {
    pub msg_id: u8,
    pub ctrl_flag: RxGlobalControlFlags,
    pub command: CommandType,
    pub mod_size: u8,
    pub seq_size: u16,
    pub seq_div: u16,
    pub(crate) mod_data: [u8; MOD_FRAME_SIZE],
}

bitflags! {
pub struct RxGlobalControlFlags : u8 {
    const NONE = 0;
    const LOOP_BEGIN = 1;
    const LOOP_END = 1 << 1;
    //
    const SILENT = 1 << 3;
    const FORCE_FAN = 1 << 4;
    const SEQ_MODE = 1 << 5;
    const SEQ_BEGIN = 1 << 6;
    const SEQ_END = 1 << 7;
}
}

impl RxGlobalHeader {
    pub fn new_with_cmd(command: CommandType) -> RxGlobalHeader {
        RxGlobalHeader {
            msg_id: command as u8,
            ctrl_flag: RxGlobalControlFlags::NONE,
            command,
            mod_size: 0,
            seq_size: 0,
            seq_div: 0,
            mod_data: [0x00; MOD_FRAME_SIZE],
        }
    }

    pub fn new_op(ctrl_flag: RxGlobalControlFlags, data: &[u8]) -> RxGlobalHeader {
        MSG_ID.fetch_add(1, atomic::Ordering::SeqCst);
        MSG_ID.compare_and_swap(
            OP_MODE_MSG_ID_MAX + 1,
            OP_MODE_MSG_ID_MIN,
            atomic::Ordering::SeqCst,
        );

        let mut data_array = [0x00; MOD_FRAME_SIZE];
        data_array[..data.len()].clone_from_slice(&data[..]);

        RxGlobalHeader {
            msg_id: MSG_ID.load(atomic::Ordering::SeqCst),
            ctrl_flag,
            command: CommandType::CmdOp,
            mod_size: data.len() as u8,
            seq_size: 0,
            seq_div: 0,
            mod_data: data_array,
        }
    }

    pub fn new_seq(ctrl_flag: RxGlobalControlFlags, seq_size: u16, seq_div: u16) -> RxGlobalHeader {
        MSG_ID.fetch_add(1, atomic::Ordering::SeqCst);
        MSG_ID.compare_and_swap(
            OP_MODE_MSG_ID_MAX + 1,
            OP_MODE_MSG_ID_MIN,
            atomic::Ordering::SeqCst,
        );

        RxGlobalHeader {
            msg_id: MSG_ID.load(atomic::Ordering::SeqCst),
            ctrl_flag,
            command: CommandType::CmdSeqMode,
            mod_size: 0,
            seq_size,
            seq_div,
            mod_data: [0x00; MOD_FRAME_SIZE],
        }
    }
}

impl fmt::Debug for RxGlobalHeader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            r"RxGlobalHeader {{
    msg_id: {},
    ctrl_flag: {:?},
    command: {:?},
    mod_size: {},
    seq_size: {},
    seq_div: {},
    mod_data: {:?},
}}",
            self.msg_id,
            self.ctrl_flag,
            self.command,
            self.mod_size,
            self.seq_size,
            self.seq_div,
            &self.mod_data[..],
        )
    }
}
