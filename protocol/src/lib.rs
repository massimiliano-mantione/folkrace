pub mod map;
pub mod protocol;
use vek::{Vec3,Quaternion};

pub type V3 = Vec3<f32>;
pub type Q = Quaternion<f32>;

#[cfg(test)]
mod test;
