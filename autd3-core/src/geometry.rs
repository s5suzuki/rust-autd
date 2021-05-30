/*
 * File: geometry.rs
 * Project: src
 * Created Date: 24/05/2021
 * Author: Shun Suzuki
 * -----
 * Last Modified: 30/05/2021
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2021 Hapis Lab. All rights reserved.
 *
 */

pub type Vector3 = nalgebra::Vector3<f64>;
pub type Vector4 = nalgebra::Vector4<f64>;
pub type Quaternion = nalgebra::Quaternion<f64>;
pub type UnitQuaternion = nalgebra::UnitQuaternion<f64>;
pub type Matrix4x4 = nalgebra::Matrix4<f64>;

use crate::hardware_defined::is_missing_transducer;

use super::hardware_defined::{NUM_TRANS_IN_UNIT, NUM_TRANS_X, NUM_TRANS_Y, TRANS_SPACING_MM};

struct Device {
    global_trans_positions: Vec<Vector3>,
    x_direction: Vector3,
    y_direction: Vector3,
    z_direction: Vector3,
}

impl Device {
    pub fn new(position: Vector3, rotation: UnitQuaternion) -> Device {
        let rot_mat: Matrix4x4 = From::from(rotation);
        let trans_mat = rot_mat.append_translation(&position);
        let x_direction = Self::get_direction(Vector3::x(), rotation);
        let y_direction = Self::get_direction(Vector3::y(), rotation);
        let z_direction = Self::get_direction(Vector3::z(), rotation);

        let mut global_trans_positions = Vec::with_capacity(NUM_TRANS_IN_UNIT);
        for y in 0..NUM_TRANS_Y {
            for x in 0..NUM_TRANS_X {
                if !is_missing_transducer(x, y) {
                    let local_pos = Vector4::new(
                        x as f64 * TRANS_SPACING_MM,
                        y as f64 * TRANS_SPACING_MM,
                        0.,
                        1.,
                    );
                    global_trans_positions.push(convert_to_vec3(trans_mat * local_pos));
                }
            }
        }

        Device {
            global_trans_positions,
            x_direction,
            y_direction,
            z_direction,
        }
    }

    fn get_direction(dir: Vector3, rotation: UnitQuaternion) -> Vector3 {
        let dir: UnitQuaternion = UnitQuaternion::from_quaternion(Quaternion::from_imag(dir));
        (rotation * dir * rotation.conjugate()).imag().normalize()
    }
}

#[derive(Default)]
pub struct Geometry {
    devices: Vec<Device>,
    pub wavelength: f64,
    pub attenuation: f64,
}

impl Geometry {
    pub fn new() -> Self {
        Self {
            devices: vec![],
            wavelength: 8.5,
            attenuation: 0.0,
        }
    }

    /// Add device to the geometry.
    ///
    /// Use this method to specify the device geometry in order of proximity to the master.
    /// Call this method or [add_device_quaternion](#method.add_device_quaternion) as many times as the number of AUTDs connected to the master.
    ///
    /// # Arguments
    ///
    /// * `pos` - Global position of AUTD.
    /// * `rot` - ZYZ Euler angles.
    ///
    /// # Example
    ///
    /// ```
    /// use std::f64::consts::PI;
    /// use autd3_core::geometry::{Vector3, Geometry};
    ///
    /// let mut geometry = Geometry::new();
    ///
    /// geometry.add_device(Vector3::zeros(), Vector3::zeros());
    /// geometry.add_device(Vector3::new(192., 0., 0.), Vector3::new(-PI, 0., 0.));
    /// ```
    pub fn add_device(&mut self, position: Vector3, euler_angles: Vector3) {
        let q = UnitQuaternion::from_axis_angle(&Vector3::z_axis(), euler_angles.x)
            * UnitQuaternion::from_axis_angle(&Vector3::y_axis(), euler_angles.y)
            * UnitQuaternion::from_axis_angle(&Vector3::z_axis(), euler_angles.z);
        self.add_device_quaternion(position, q)
    }

    /// Add device to the geometry.
    ///
    /// Use this method to specify the device geometry in order of proximity to the master.
    /// Call this method or [add_device](#method.add_device) as many times as the number of AUTDs connected to the master.
    ///
    /// # Arguments
    ///
    /// * `pos` - Global position of AUTD.
    /// * `rot` - Rotation quaternion.
    ///
    pub fn add_device_quaternion(&mut self, position: Vector3, rotation: UnitQuaternion) {
        self.devices.push(Device::new(position, rotation));
    }

    pub fn num_devices(&self) -> usize {
        self.devices.len()
    }

    pub fn position_by_global_idx(&self, global_transducer_idx: usize) -> Vector3 {
        let local_trans_idx = global_transducer_idx % NUM_TRANS_IN_UNIT;
        let device_idx = global_transducer_idx / NUM_TRANS_IN_UNIT;
        self.position_by_local_idx(device_idx, local_trans_idx)
    }

    pub fn position_by_local_idx(&self, device_idx: usize, local_trans_idx: usize) -> Vector3 {
        self.devices[device_idx].global_trans_positions[local_trans_idx]
    }

    pub fn local_position(&self, device_idx: usize, global_position: Vector3) -> Vector3 {
        let device = &self.devices[device_idx];
        let local_origin = device.global_trans_positions[0];
        let x_dir = device.x_direction;
        let y_dir = device.y_direction;
        let z_dir = device.z_direction;
        let rv = global_position - local_origin;
        Vector3::new(rv.dot(&x_dir), rv.dot(&y_dir), rv.dot(&z_dir))
    }

    pub fn direction(&self, device_idx: usize) -> Vector3 {
        self.devices[device_idx].z_direction
    }
}

fn convert_to_vec3(v: Vector4) -> Vector3 {
    Vector3::new(v.x, v.y, v.z)
}
