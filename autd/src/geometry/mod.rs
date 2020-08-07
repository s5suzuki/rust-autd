pub type Vector3 = na::Vec3<f64>;
pub type Vector4 = na::Vec4<f64>;
pub type Quaternion = na::Quaternion<f64>;
pub type UnitQuaternion = na::UnitQuaternion<f64>;
pub type Matrix4x4 = na::Matrix4<f64>;

use crate::consts::{NUM_TRANS_IN_UNIT, NUM_TRANS_X, NUM_TRANS_Y, TRANS_SIZE};

struct Device {
    device_id: usize,
    global_trans_positions: Vec<Vector3>,
    x_direction: Vector3,
    y_direction: Vector3,
    z_direction: Vector3,
}

impl Device {
    pub fn new(device_id: usize, position: Vector3, rotation: UnitQuaternion) -> Device {
        let rot_mat: Matrix4x4 = From::from(rotation);
        let trans_mat = rot_mat.append_translation(&position);
        let x_direction = Self::get_direction(Vector3::x(), rotation);
        let y_direction = Self::get_direction(Vector3::y(), rotation);
        let z_direction = Self::get_direction(Vector3::z(), rotation);

        let mut global_trans_positions = Vec::with_capacity(NUM_TRANS_IN_UNIT);
        for y in 0..NUM_TRANS_Y {
            for x in 0..NUM_TRANS_X {
                if !is_missing_transducer(x, y) {
                    let local_pos =
                        Vector4::new(x as f64 * TRANS_SIZE, y as f64 * TRANS_SIZE, 0., 1.);
                    global_trans_positions.push(convert_to_vec3(trans_mat * local_pos));
                }
            }
        }

        Device {
            device_id,
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
}

impl Geometry {
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
    /// use autd::geometry::{Vector3, Geometry};
    ///
    /// let mut geometry: Geometry = Default::default();
    ///
    /// geometry.add_device(Vector3::zeros(), Vector3::zeros());
    /// geometry.add_device(Vector3::new(192., 0., 0.), Vector3::new(-PI, 0., 0.));
    /// ```
    pub fn add_device(&mut self, position: Vector3, euler_angles: Vector3) -> usize {
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
    pub fn add_device_quaternion(&mut self, position: Vector3, rotation: UnitQuaternion) -> usize {
        let device_id = self.devices.len();
        self.devices
            .push(Device::new(device_id, position, rotation));
        device_id
    }

    pub fn del_device(&mut self, device_id: usize) {
        let mut index = 0;
        for (i, dev) in self.devices.iter().enumerate() {
            if dev.device_id == device_id {
                index = i;
                break;
            }
        }
        self.devices.remove(index);
    }

    pub fn num_devices(&self) -> usize {
        self.devices.len()
    }

    pub fn position(&self, transducer_id: usize) -> Vector3 {
        let local_trans_id = transducer_id % NUM_TRANS_IN_UNIT;
        let device = self.device(transducer_id);
        device.global_trans_positions[local_trans_id]
    }

    pub fn local_position(&self, device_id: usize, global_position: Vector3) -> Vector3 {
        let device = &self.devices[device_id];
        let local_origin = device.global_trans_positions[0];
        let x_dir = device.x_direction;
        let y_dir = device.y_direction;
        let z_dir = device.z_direction;
        let rv = global_position - local_origin;
        Vector3::new(rv.dot(&x_dir), rv.dot(&y_dir), rv.dot(&z_dir))
    }

    pub fn direction(&self, transducer_id: usize) -> Vector3 {
        let device = self.device(transducer_id);
        device.z_direction
    }

    fn device(&self, transducer_id: usize) -> &Device {
        let eid = transducer_id / NUM_TRANS_IN_UNIT;
        &self.devices[eid]
    }
}

pub fn is_missing_transducer(x: usize, y: usize) -> bool {
    y == 1 && (x == 1 || x == 2 || x == 16)
}

fn convert_to_vec3(v: Vector4) -> Vector3 {
    Vector3::new(v.x, v.y, v.z)
}
