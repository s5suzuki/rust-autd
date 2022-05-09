/*
 * File: error.rs
 * Project: fpga
 * Created Date: 02/05/2022
 * Author: Shun Suzuki
 * -----
 * Last Modified: 05/05/2022
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2022 Hapis Lab. All rights reserved.
 *
 */

use thiserror::Error;

#[derive(Error, Debug)]
pub enum FPGAError {
    #[error(
        "Minimum is {}, but {0} is specified",
        crate::fpga::MOD_SAMPLING_FREQ_DIV_MIN
    )]
    ModFreqDivOutOfRange(u32),
    #[error(
        "Minimum is {}, but {0} is specified",
        crate::fpga::STM_SAMPLING_FREQ_DIV_MIN
    )]
    STMFreqDivOutOfRange(u32),
    #[error("Minimum is {}, but {0} is specified", crate::fpga::SILENCER_CYCLE_MIN)]
    SilencerCycleOutOfRange(u16),
    #[error("Maximum is {}, but {0} are to be sent", crate::MOD_BUF_SIZE_MAX)]
    ModulationOutOfBuffer(usize),
    #[error("Maximum is {}, but {0} are to be sent", crate::POINT_STM_BUF_SIZE_MAX)]
    PointSTMOutOfBuffer(usize),
    #[error("Maximum is {}, but {0} are to be sent", crate::GAIN_STM_BUF_SIZE_MAX)]
    GainSTMOutOfBuffer(usize),
}
