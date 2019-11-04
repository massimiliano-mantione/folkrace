use nalgebra::RealField;
use nalgebra::{UnitQuaternion, Vector3, Isometry3};
use vek::{Vec3,Quaternion};

use protocol::map::{Map,MapSectionShape};

pub type V3 = Vec3<f32>;
pub type Q = Quaternion<f32>;

pub type NaV3 = Vector3<f32>;
pub type NaQ = UnitQuaternion<f32>;
pub type ISO = Isometry3<f32>;

pub trait RotationComponents {
    fn rot_x(&self) -> f32;
    fn rot_y(&self) -> f32;
    fn rot_z(&self) -> f32;
}

impl RotationComponents for NaQ {
    fn rot_x(&self) -> f32 {
        match self.axis() {
            Some(axis) => axis.x * self.angle(),
            None => 0.0,
        }
    }
    fn rot_y(&self) -> f32 {
        match self.axis() {
            Some(axis) => axis.y * self.angle(),
            None => 0.0,
        }
    }
    fn rot_z(&self) -> f32 {
        match self.axis() {
            Some(axis) => axis.z * self.angle(),
            None => 0.0,
        }
    }
}

pub const CAR_LENGTH: f32 = 0.2;
pub const CAR_WIDTH: f32 = 0.15;
pub const CAR_MASS: f32 = 0.8;

pub const CAR_WHEEL_RADIUS: f32 = 0.035;
pub const CAR_WHEEL_THICKNESS: f32 = 0.02;
pub const CAR_WHEEL_MASS: f32 = 0.1;

pub const CAR_WHEEL_SPACE: f32 = 0.005;

pub const MAP_WALL_H: f32 = 0.15;
pub const MAP_WALL_THICKNESS: f32 = 0.01;
pub const MAP_FLOOR_THICKNESS: f32 = 0.01;

#[derive(Clone, Copy)]
pub struct Car {
    pub length: f32,
    pub width: f32,
    pub mass: f32,
    pub wheel_radius: f32,
    pub wheel_thickness: f32,
    pub wheel_mass: f32,

    pub position: V3,
    pub rotation: Q,
}

impl Car {
    pub fn new() -> Self {
        Car {
            length: CAR_LENGTH,
            width: CAR_WIDTH,
            mass: CAR_MASS,
            wheel_radius: CAR_WHEEL_RADIUS,
            wheel_thickness: CAR_WHEEL_THICKNESS,
            wheel_mass: CAR_WHEEL_MASS,

            position: V3::zero(),
            rotation: Q::zero(),
        }
    }

    pub fn body_l(&self) -> f32 {
        self.length
    }
    pub fn body_h(&self) -> f32 {
        self.wheel_radius * 2.0
    }
    pub fn body_w(&self) -> f32 {
        self.width - ((self.wheel_thickness + CAR_WHEEL_SPACE) * 2.0)
    }

    pub fn body_position(&self) -> Vector3<f32> {
        Vector3::new(0.0, self.wheel_radius / 2.0, 0.0)
    }

    pub fn laser_position(&self) -> Vector3<f32> {
        Vector3::new(
            0.0,
            self.body_position().z + (self.body_h() * 3.0 / 4.0),
            self.body_l() * 1.0 / 4.0,
        )
    }

    pub fn wheel_x(&self) -> f32 {
        (self.width - self.wheel_thickness) / 2.0
    }
    pub fn wheel_y(&self) -> f32 {
        0.0
    }
    pub fn wheel_z(&self) -> f32 {
        (self.length / 2.0) - self.wheel_radius
    }
}

#[derive(Clone, Copy)]
pub struct MapSectionSegment {
    pub center: NaV3,
    pub heading: f32,
    pub pitch: f32,
    pub length_left: f32,
    pub length_right: f32,
    pub width_start: f32,
    pub width_end: f32,
    pub is_lighter: bool,
}

#[derive(Clone, Copy)]
pub struct MapSectionBox {
    pub center: NaV3,
    pub rotation: NaQ,
    pub width: f32,
    pub length: f32,
    pub height: f32,
}

impl MapSectionSegment {
    pub fn new(
        center: NaV3,
        heading: f32,
        pitch: f32,
        length_left: f32,
        length_right: f32,
        width_start: f32,
        width_end: f32,
        is_lighter: bool,
    ) -> Self {
        MapSectionSegment {
            center,
            heading,
            pitch,
            length_left,
            length_right,
            width_start,
            width_end,
            is_lighter,
        }
    }

    pub fn rotation(&self) -> NaQ {
        let rotation_x = NaQ::from_axis_angle(&NaV3::x_axis(), -self.pitch);
        let rotation_y = NaQ::from_axis_angle(&NaV3::y_axis(), self.heading);
        rotation_y * rotation_x
    }

    pub fn max_length(&self) -> f32 {
        f32::max(self.length_left, self.length_right)
    }

    pub fn max_width(&self) -> f32 {
        f32::max(self.width_start, self.width_end)
    }

    fn slope_wall_extra_length(&self) -> f32 {
        1.0 * self.pitch.abs().tan() * MAP_WALL_H
    }

    pub fn floor_box(&self) -> MapSectionBox {
        let length = self.max_length();
        let width = self.max_width();
        let height = MAP_FLOOR_THICKNESS;
        MapSectionBox {
            center: self.center,
            width,
            length,
            height,
            rotation: self.rotation(),
        }
    }

    pub fn left_box(&self) -> MapSectionBox {
        let length = self.length_left;
        let width = MAP_WALL_THICKNESS;
        let height = MAP_WALL_H;
        let displacement = Vector3::new(0.0, 0.0, self.max_width() / 2.0);
        let displacement_rotation: NaQ =
            NaQ::from_axis_angle(&NaV3::y_axis(), self.rotation().rot_y() + f32::frac_pi_2());
        let displacement = displacement_rotation.transform_vector(&displacement);
        let displacement =
            displacement + Vector3::new(0.0, (MAP_WALL_H / 2.0) - MAP_FLOOR_THICKNESS, 0.0);
        MapSectionBox {
            center: self.center + displacement,
            width,
            length: length + self.slope_wall_extra_length(),
            height,
            rotation: self.rotation(),
        }
    }

    pub fn right_box(&self) -> MapSectionBox {
        let length = self.length_right;
        let width = MAP_WALL_THICKNESS;
        let height = MAP_WALL_H;
        let displacement = Vector3::new(0.0, 0.0, self.max_width() / 2.0);
        let displacement_rotation: NaQ =
            NaQ::from_axis_angle(&NaV3::y_axis(), self.rotation().rot_y() - f32::frac_pi_2());
        let displacement = displacement_rotation.transform_vector(&displacement);
        let displacement =
            displacement + Vector3::new(0.0, (MAP_WALL_H / 2.0) - MAP_FLOOR_THICKNESS, 0.0);
        MapSectionBox {
            center: self.center + displacement,
            width,
            length: length + self.slope_wall_extra_length(),
            height,
            rotation: self.rotation(),
        }
    }
}

fn v3(v: V3) -> NaV3 {
    Vector3::new(v.x, v.y, v.z)
}

fn lerp(v1: f32, v2: f32, interval: f32) -> f32 {
    v1 + ((v2 - v1) * interval)
}

pub fn map_segmentation(map: &Map) -> Vec<MapSectionSegment> {
    let mut segments = vec![];

    for i in 0..map.length {
        let section = &map[i];
        match section.shape {
            MapSectionShape::Straigth(s) => {
                segments.push(MapSectionSegment::new(
                    v3(section.center),
                    section.heading_start,
                    0.0,
                    s.length,
                    s.length,
                    section.width_start,
                    section.width_end,
                    true,
                ));
            }
            MapSectionShape::Slope(s) => {
                segments.push(MapSectionSegment::new(
                    v3(section.center),
                    section.heading_start,
                    (s.height / s.length).atan(),
                    (s.length.powi(2) + s.height.powi(2)).sqrt(),
                    (s.length.powi(2) + s.height.powi(2)).sqrt(),
                    section.width_start,
                    section.width_end,
                    true,
                ));
            }
            MapSectionShape::Turn(s) => {
                let steps = (s.turning_angle.abs().to_degrees() / 15.0) as i32;
                let steps = if steps % 2 == 0 { steps + 1 } else { steps };
                let half_steps = steps * 2;
                let half_interval = 1.0 / half_steps as f32;
                let angle_half_interval = (s.turning_angle * half_interval).abs();
                let inner_length_start = (s.radius_start - (section.width_start / 2.0))
                    * angle_half_interval.sin()
                    * 2.0;
                let outer_length_start = (s.radius_start + (section.width_start / 2.0))
                    * angle_half_interval.sin()
                    * 2.0;
                let inner_length_end =
                    (s.radius_end - (section.width_end / 2.0)) * angle_half_interval.sin() * 2.0;
                let outer_length_end =
                    (s.radius_end + (section.width_end / 2.0)) * angle_half_interval.sin() * 2.0;
                let (left_length_start, right_length_start, left_length_end, right_length_end) =
                    if s.turning_angle > 0.0 {
                        (
                            inner_length_start,
                            outer_length_start,
                            inner_length_end,
                            outer_length_end,
                        )
                    } else {
                        (
                            outer_length_start,
                            inner_length_start,
                            outer_length_end,
                            inner_length_end,
                        )
                    };
                let center_to_start = (section.start - section.center).normalized();
                let mut is_lighter = false;
                for i in 0..steps {
                    let interval = half_interval * (i as f32 * 2.0 + 1.0);
                    let angle = lerp(0.0, s.turning_angle, interval);
                    let rot = NaQ::from_axis_angle(&NaV3::y_axis(), angle);
                    let center_to_segment_center = rot.transform_vector(&v3(center_to_start))
                        * lerp(s.radius_start, s.radius_end, interval);
                    let segment_center = v3(section.center) + center_to_segment_center;
                    segments.push(MapSectionSegment::new(
                        segment_center,
                        section.heading_start + angle,
                        0.0,
                        lerp(left_length_start, left_length_end, interval),
                        lerp(right_length_start, right_length_end, interval),
                        lerp(section.width_start, section.width_end, interval),
                        lerp(section.width_start, section.width_end, interval),
                        is_lighter,
                    ));
                    is_lighter = !is_lighter;
                }
            }
        }
    }

    segments
}
