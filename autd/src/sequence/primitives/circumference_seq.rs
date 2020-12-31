/*
 * File: circumference_seq.rs
 * Project: src
 * Created Date: 30/06/2020
 * Author: Shun Suzuki
 * -----
 * Last Modified: 30/12/2020
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2020 Hapis Lab. All rights reserved.
 *
 */

use super::super::PointSequence;

use crate::geometry::Vector3;
use crate::{Float, PI};

pub struct CircumSeq {}

impl CircumSeq {
    pub fn create(center: Vector3, normal: Vector3, radius: Float, n: usize) -> PointSequence {
        let normal = normal.normalize();
        let n1 = Self::get_orthogonal(&normal).normalize();
        let n2 = normal.cross(&n1).normalize();

        let mut control_points: Vec<Vector3> = Vec::with_capacity(n);
        for i in 0..n {
            let theta = 2.0 * PI / n as Float * i as Float;
            let x = n1 * radius * theta.cos();
            let y = n2 * radius * theta.sin();
            control_points.push(center + x + y);
        }
        PointSequence::with_control_points(control_points)
    }

    fn get_orthogonal(v: &Vector3) -> Vector3 {
        let mut a = Vector3::x();
        if v.angle(&a) < PI / 2.0 {
            a = Vector3::y();
        }
        v.cross(&a)
    }
}
