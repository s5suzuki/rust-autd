/*
 * File: error_handler.rs
 * Project: ecat_thread
 * Created Date: 03/05/2022
 * Author: Shun Suzuki
 * -----
 * Last Modified: 05/05/2022
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2022 Hapis Lab. All rights reserved.
 *
 */

use crate::native_methods::*;

pub struct EcatErrorHandler<F: Fn(&str)> {
    pub error_handle: Option<F>,
}

impl<F: Fn(&str)> EcatErrorHandler<F> {
    pub fn handle(&self) -> bool {
        unsafe {
            ec_group[0].docheckstate = 0;
            ec_readstate();
            let mut msg = String::new();
            ec_slave
                .iter_mut()
                .enumerate()
                .take(ec_slavecount as usize + 1)
                .skip(1)
                .for_each(|(i, slave)| {
                    if slave.state != ec_state_EC_STATE_OPERATIONAL as _ {
                        ec_group[0].docheckstate = 1;
                        if slave.state
                            == ec_state_EC_STATE_SAFE_OP as u16 + ec_state_EC_STATE_ERROR as u16
                        {
                            msg.push_str(&format!(
                                "ERROR : slave {} is in SAFE_OP + ERROR, attempting ack\n",
                                i
                            ));
                            slave.state =
                                ec_state_EC_STATE_SAFE_OP as u16 + ec_state_EC_STATE_ACK as u16;
                            ec_writestate(i as _);
                        } else if slave.state == ec_state_EC_STATE_SAFE_OP as _ {
                            msg.push_str(&format!(
                                "ERROR : slave {} is in SAFE_OP, change to OPERATIONAL\n",
                                i
                            ));
                            slave.state = ec_state_EC_STATE_OPERATIONAL as _;
                            ec_writestate(i as _);
                        } else if slave.state > ec_state_EC_STATE_NONE as _ {
                            if ec_reconfig_slave(i as _, 500) != 0 {
                                slave.islost = 0;
                                msg.push_str(&format!("MESSAGE : slave {} reconfigured\n", i));
                            }
                        } else if slave.islost == 0 {
                            ec_statecheck(
                                i as _,
                                ec_state_EC_STATE_OPERATIONAL as _,
                                EC_TIMEOUTRET as _,
                            );
                            if slave.state == ec_state_EC_STATE_NONE as _ {
                                slave.islost = 1;
                                msg.push_str(&format!("ERROR : slave {} lost\n", i));
                            }
                        }
                    }
                    if slave.islost != 0 {
                        if slave.state == ec_state_EC_STATE_NONE as _ {
                            if ec_recover_slave(i as _, 500) != 0 {
                                slave.islost = 0;
                                msg.push_str(&format!("MESSAGE : slave {} recovered\n", i));
                            }
                        } else {
                            slave.islost = 0;
                            msg.push_str(&format!("MESSAGE : slave {} found\n", i));
                        }
                    }
                });

            if ec_group[0].docheckstate == 0 {
                return true;
            }

            if let Some(f) = &self.error_handle {
                for slave in ec_slave.iter().take(ec_slavecount as usize + 1).skip(1) {
                    if slave.islost != 0 {
                        f(&msg);
                        return false;
                    }
                }
            }
            true
        }
    }
}