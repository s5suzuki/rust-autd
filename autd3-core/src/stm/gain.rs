/*
 * File: gain.rs
 * Project: stm
 * Created Date: 05/05/2022
 * Author: Shun Suzuki
 * -----
 * Last Modified: 07/05/2022
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2022 Hapis Lab. All rights reserved.
 *
 */

use crate::{
    gain::Gain,
    geometry::{Geometry, LegacyTransducer, NormalTransducer, Transducer},
    interface::{DatagramBody, Empty, Filled, Sendable},
};

use anyhow::{Ok, Result};
use autd3_driver::{TxDatagram, FPGA_CLK_FREQ, STM_SAMPLING_FREQ_DIV_MIN};

use super::STM;

pub struct GainSTM<T: Transducer> {
    gains: Vec<T::D>,
    sample_freq_div: u32,
    next_duty: bool,
    sent: usize,
}

impl<T: Transducer> GainSTM<T> {
    pub fn new() -> Self {
        Self {
            gains: vec![],
            sample_freq_div: 4096,
            next_duty: false,
            sent: 0,
        }
    }

    pub fn add_gain<G: Gain<T>>(&mut self, gain: G, geometry: &Geometry<T>) -> Result<()> {
        if self.gains.len() + 1 > autd3_driver::GAIN_STM_BUF_SIZE_MAX {
            return Err(autd3_driver::FPGAError::GainSTMOutOfBuffer(self.gains.len() + 1).into());
        }

        let mut gain = gain;

        gain.build(geometry)?;

        let drives = gain.take_drives();

        self.gains.push(drives);
        Ok(())
    }

    pub fn size(&self) -> usize {
        self.gains.len()
    }
}

impl<T: Transducer> Default for GainSTM<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl DatagramBody<LegacyTransducer> for GainSTM<LegacyTransducer> {
    fn init(&mut self) -> Result<()> {
        self.sent = 0;
        Ok(())
    }

    fn pack(
        &mut self,
        msg_id: u8,
        _geometry: &Geometry<LegacyTransducer>,
        tx: &mut TxDatagram,
    ) -> Result<()> {
        if DatagramBody::<LegacyTransducer>::is_finished(self) {
            return Ok(());
        }

        let is_first_frame = self.sent == 0;
        let is_last_frame = self.sent + 1 == self.gains.len() + 1;

        if is_first_frame {
            autd3_driver::gain_stm_legacy(
                msg_id,
                &[],
                is_first_frame,
                self.sample_freq_div,
                is_last_frame,
                tx,
            )?;
            self.sent += 1;
            return Ok(());
        }

        autd3_driver::gain_stm_legacy(
            msg_id,
            &self.gains[self.sent - 1].data,
            is_first_frame,
            self.sample_freq_div,
            is_last_frame,
            tx,
        )?;
        self.sent += 1;
        Ok(())
    }

    fn is_finished(&self) -> bool {
        self.sent == self.gains.len() + 1
    }
}

impl DatagramBody<NormalTransducer> for GainSTM<NormalTransducer> {
    fn init(&mut self) -> Result<()> {
        self.sent = 0;
        self.next_duty = false;
        Ok(())
    }

    fn pack(
        &mut self,
        msg_id: u8,
        _geometry: &Geometry<NormalTransducer>,
        tx: &mut TxDatagram,
    ) -> Result<()> {
        if DatagramBody::<NormalTransducer>::is_finished(self) {
            return Ok(());
        }

        let is_first_frame = self.sent == 0;
        let is_last_frame = self.sent + 1 == self.gains.len() * 2 + 1;

        if is_first_frame {
            autd3_driver::gain_stm_normal_phase(
                msg_id,
                &[],
                is_first_frame,
                self.sample_freq_div,
                tx,
            )?;
            self.sent += 1;
            return Ok(());
        }

        if !self.next_duty {
            autd3_driver::gain_stm_normal_phase(
                msg_id,
                &self.gains[(self.sent - 1) / 2].phases,
                is_first_frame,
                self.sample_freq_div,
                tx,
            )?;
            self.next_duty = true;
        } else {
            autd3_driver::gain_stm_normal_duty(
                msg_id,
                &self.gains[(self.sent - 1) / 2].duties,
                is_first_frame,
                self.sample_freq_div,
                is_last_frame,
                tx,
            )?;
            self.next_duty = false;
        }

        self.sent += 1;

        Ok(())
    }

    fn is_finished(&self) -> bool {
        self.sent == self.gains.len() * 2 + 1
    }
}

impl Sendable<LegacyTransducer> for GainSTM<LegacyTransducer> {
    type H = Empty;
    type B = Filled;

    fn init(&mut self) -> Result<()> {
        DatagramBody::<LegacyTransducer>::init(self)
    }

    fn pack(
        &mut self,
        msg_id: u8,
        geometry: &Geometry<LegacyTransducer>,
        tx: &mut TxDatagram,
    ) -> Result<()> {
        DatagramBody::<LegacyTransducer>::pack(self, msg_id, geometry, tx)
    }

    fn is_finished(&self) -> bool {
        DatagramBody::<LegacyTransducer>::is_finished(self)
    }
}

impl Sendable<NormalTransducer> for GainSTM<NormalTransducer> {
    type H = Empty;
    type B = Filled;

    fn init(&mut self) -> Result<()> {
        DatagramBody::<NormalTransducer>::init(self)
    }

    fn pack(
        &mut self,
        msg_id: u8,
        geometry: &Geometry<NormalTransducer>,
        tx: &mut TxDatagram,
    ) -> Result<()> {
        DatagramBody::<NormalTransducer>::pack(self, msg_id, geometry, tx)
    }

    fn is_finished(&self) -> bool {
        DatagramBody::<NormalTransducer>::is_finished(self)
    }
}

impl<T: Transducer> STM for GainSTM<T> {
    fn set_freq(&mut self, freq: f64) -> f64 {
        let sample_freq = self.size() as f64 * freq;
        let div = ((FPGA_CLK_FREQ as f64 / sample_freq) as u32)
            .clamp(STM_SAMPLING_FREQ_DIV_MIN, u32::MAX);
        self.sample_freq_div = div;
        STM::freq(self)
    }

    fn freq(&self) -> f64 {
        STM::sampling_freq(self) / self.size() as f64
    }

    fn sampling_freq(&self) -> f64 {
        FPGA_CLK_FREQ as f64 / self.sample_freq_div as f64
    }

    fn set_sampling_freq_div(&mut self, freq_div: u32) {
        self.sample_freq_div = freq_div;
    }

    fn sampling_freq_div(&mut self) -> u32 {
        self.sample_freq_div
    }
}
