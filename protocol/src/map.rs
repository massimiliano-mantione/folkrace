use hal::{Angle, LinearDimension};
use crate::protocol::{ProtocolAngle, ProtocolLinearDimension, ProtocolMapSectionData};
use crate::{Q, V3};
use core::f32::consts::*;

#[derive(Clone, Copy)]
/// Straigth map section
pub struct MapSectionStraigth {
    // Length
    pub length: LinearDimension,
}

#[derive(Clone, Copy)]
/// Sloping map section
pub struct MapSectionSlope {
    // Length
    pub length: LinearDimension,
    // Slope height (negative if descending)
    pub height: LinearDimension,
}

#[derive(Clone, Copy)]
/// Turning map section
pub struct MapSectionTurn {
    // Starting radius
    pub radius_start: LinearDimension,
    // Ending radius
    pub radius_end: LinearDimension,
    // Turning angle
    pub turning_angle: Angle,
}

#[derive(Clone, Copy)]
/// Map section shape
pub enum MapSectionShape {
    Straigth(MapSectionStraigth),
    Slope(MapSectionSlope),
    Turn(MapSectionTurn),
}

#[derive(Clone, Copy)]
/// Map section
pub struct MapSection {
    /// Section shape
    pub shape: MapSectionShape,
    /// Starting width
    pub width_start: LinearDimension,
    /// Ending width
    pub width_end: LinearDimension,

    /// Center of section start
    pub start: V3,
    /// Center of section end
    pub end: V3,
    /// Either center of section (for straight and slopes) or center of rotation (for turns)
    pub center: V3,
    /// Starting heading
    pub heading_start: Angle,
    /// Ending heading
    pub heading_end: Angle,
}

fn dim_from_proto(dim: ProtocolLinearDimension) -> LinearDimension {
    dim as LinearDimension / 1000.0
}
fn ang_from_proto(ang: ProtocolAngle) -> Angle {
    -(ang as Angle) * 3.1415 / 180.0
}

fn normalize_angle(angle: f32) -> f32 {
    let mut angle = angle;
    while angle.to_degrees() > PI {
        angle -= PI * 2.0;
    }
    while angle < -PI {
        angle += PI * 2.0;
    }
    angle
}

impl MapSection {
    /// Build new section with config data
    pub fn new(
        shape: MapSectionShape,
        width_start: LinearDimension,
        width_end: LinearDimension,
    ) -> Self {
        MapSection {
            shape,
            width_start,
            width_end,

            start: V3::zero(),
            end: V3::zero(),
            center: V3::zero(),
            heading_start: 0.0,
            heading_end: 0.0,
        }
    }

    /// Check if section is valid
    pub fn is_valid(&self) -> bool {
        if self.width_start <= 0.0 || self.width_end <= 0.0 {
            return false;
        }
        match self.shape {
            MapSectionShape::Straigth(s) => {
                if s.length <= 0.0 {
                    return false;
                }
            }
            MapSectionShape::Slope(s) => {
                if s.length <= 0.0 {
                    return false;
                }
                if s.height == 0.0 {
                    return false;
                }
            }
            MapSectionShape::Turn(s) => {
                if s.radius_start <= 0.0 {
                    return false;
                }
                if s.radius_end <= 0.0 {
                    return false;
                }
            }
        }
        return true;
    }

    pub fn from_protocol_data(data: &ProtocolMapSectionData) -> Self {
        match data {
            ProtocolMapSectionData::Straight(s) => MapSection::new(
                MapSectionShape::Straigth(MapSectionStraigth {
                    length: dim_from_proto(s.length),
                }),
                dim_from_proto(s.width_start),
                dim_from_proto(s.width_end),
            ),
            ProtocolMapSectionData::TurnRight(s) => MapSection::new(
                MapSectionShape::Turn(MapSectionTurn {
                    radius_start: dim_from_proto(s.radius_start),
                    radius_end: dim_from_proto(s.radius_end),
                    turning_angle: ang_from_proto(s.angle),
                }),
                dim_from_proto(s.width_start),
                dim_from_proto(s.width_end),
            ),
            ProtocolMapSectionData::TurnLeft(s) => MapSection::new(
                MapSectionShape::Turn(MapSectionTurn {
                    radius_start: dim_from_proto(s.radius_start),
                    radius_end: dim_from_proto(s.radius_end),
                    turning_angle: -ang_from_proto(s.angle),
                }),
                dim_from_proto(s.width_start),
                dim_from_proto(s.width_end),
            ),
            ProtocolMapSectionData::SlopeUp(s) => MapSection::new(
                MapSectionShape::Slope(MapSectionSlope {
                    length: dim_from_proto(s.length),
                    height: dim_from_proto(s.height),
                }),
                dim_from_proto(s.width_start),
                dim_from_proto(s.width_end),
            ),
            ProtocolMapSectionData::SlopeDown(s) => MapSection::new(
                MapSectionShape::Slope(MapSectionSlope {
                    length: dim_from_proto(s.length),
                    height: -dim_from_proto(s.height),
                }),
                dim_from_proto(s.width_start),
                dim_from_proto(s.width_end),
            ),
        }
    }

    fn compute_end_geometry(&self) -> (V3, f32, V3) {
        match self.shape {
            MapSectionShape::Straigth(s) => {
                let rot = Q::rotation_y(self.heading_start);
                let delta = rot * V3::unit_z() * s.length;
                let center = self.start + (delta / 2.0);
                (self.start + delta, self.heading_start, center)
            }
            MapSectionShape::Turn(s) => {
                let dir_front = Q::rotation_y(self.heading_start) * V3::unit_z();
                let dir_to_center = if s.turning_angle > 0.0 {
                    Q::rotation_y(FRAC_PI_2) * dir_front
                } else {
                    Q::rotation_y(-FRAC_PI_2) * dir_front
                };
                let center = self.start + (dir_to_center * s.radius_start);
                let dir_from_center_to_start = -dir_to_center;
                let from_center_to_end =
                    Q::rotation_y(s.turning_angle) * (dir_from_center_to_start * s.radius_end);
                (
                    center + from_center_to_end,
                    normalize_angle(self.heading_start + s.turning_angle),
                    center,
                )
            }
            MapSectionShape::Slope(s) => {
                let rot = Q::rotation_y(self.heading_start);
                let delta_flat = rot * V3::unit_z() * s.length;
                let delta_height = V3::unit_y() * s.height;
                let delta = delta_flat + delta_height;
                let center = self.start + (delta / 2.0);
                (self.start + delta, self.heading_start, center)
            }
        }
    }
}

pub const MAP_SECTIONS_MAX_COUNT: usize = 20;

#[derive(Clone, Copy)]
/// Description of a map
pub struct Map {
    /// Number of used sections
    pub length: usize,
    /// Sections (index zero is the starting one)
    pub sections: [MapSection; MAP_SECTIONS_MAX_COUNT],
}

const EMPTY_SECTION: MapSection = MapSection {
    shape: MapSectionShape::Straigth(MapSectionStraigth { length: 0.0 }),
    width_start: 0.0,
    width_end: 0.0,
    start: V3 {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    },
    end: V3 {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    },
    center: V3 {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    },
    heading_start: 0.0,
    heading_end: 0.0,
};

impl std::ops::Index<usize> for Map {
    type Output = MapSection;

    fn index(&self, i: usize) -> &Self::Output {
        &self.sections[self.fix_index(i)]
    }
}

impl Map {
    /// Create an empty (invalid) map
    pub fn new() -> Self {
        Map {
            length: 0,
            sections: [EMPTY_SECTION; MAP_SECTIONS_MAX_COUNT],
        }
    }

    /// Reset map to an an empty (invalid) state
    pub fn reset(&mut self) {
        self.length = 0;
        for i in 0..MAP_SECTIONS_MAX_COUNT {
            self.sections[i] = EMPTY_SECTION;
        }
    }

    /// Configure section at index
    pub fn configure_section(&mut self, index: usize, section: &MapSection) {
        self.sections[index] = *section;
    }

    /// Complete configuration after all sections have been defined
    pub fn complete_configuration(&mut self) {
        self.length = 0;
        for i in 0..MAP_SECTIONS_MAX_COUNT {
            if self.sections[i].is_valid() {
                self.length = i + 1;
            }
        }
        if self.is_valid() {
            let mut start = V3::zero();
            let mut heading_start = 0.0;
            for i in 0..self.length {
                self.sections[i].start = start;
                self.sections[i].heading_start = heading_start;
                let (end, heading_end, center) = self.sections[i].compute_end_geometry();
                self.sections[i].end = end;
                self.sections[i].heading_end = heading_end;
                self.sections[i].center = center;
                start = end;
                heading_start = heading_end;
            }
        }
    }

    /// Check if map is valid
    pub fn is_valid(&self) -> bool {
        if self.length == 0 {
            return false;
        }
        for i in 0..self.length {
            if !self.sections[i].is_valid() {
                return false;
            }
        }
        return true;
    }

    /// Make sure section index is inside map (wrap it if needed)
    pub fn fix_index(&self, index: usize) -> usize {
        if self.length > 0 {
            index % self.length
        } else {
            index
        }
    }

    /// Compute next section index
    pub fn next_index(&self, index: usize) -> usize {
        self.fix_index(index + 1)
    }

    /// Compute previous section index
    pub fn previous_index(&self, index: usize) -> usize {
        if index > 0 {
            index - 1
        } else {
            self.length - 1
        }
    }
}
