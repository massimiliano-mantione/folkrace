use nalgebra::{Isometry3, Point3, Translation3, UnitQuaternion, Vector3};

use ncollide3d::shape::{Ball, Cuboid, ShapeHandle};
use nphysics3d::force_generator::DefaultForceGeneratorSet;
use nphysics3d::joint::DefaultJointConstraintSet;
use nphysics3d::object::{BodyPartHandle, DefaultBodyHandle, DefaultBodySet, DefaultColliderSet};
use nphysics3d::world::{DefaultGeometricalWorld, DefaultMechanicalWorld};

use nphysics3d::joint::{FixedJoint, FreeJoint, RevoluteJoint};
use nphysics3d::object::{ColliderDesc, Ground, MultibodyDesc};

use map::*;
use protocol::map::Map;

pub struct SimulatedWorld {
    mechanical_world: DefaultMechanicalWorld<f32>,
    geometrical_world: DefaultGeometricalWorld<f32>,
    bodies: DefaultBodySet<f32>,
    colliders: DefaultColliderSet<f32>,
    joint_constraints: DefaultJointConstraintSet<f32>,
    force_generators: DefaultForceGeneratorSet<f32>,

    ground: DefaultBodyHandle,
    ground_part_count: usize,

    car: DefaultBodyHandle,

    car_part_id_body: usize,
    car_part_id_bl: usize,
    car_part_id_br: usize,
    car_part_id_fl: usize,
    car_part_id_fr: usize,
    car_motor_power_bl: f32,
    car_motor_power_br: f32,
    car_motor_power_fl: f32,
    car_motor_power_fr: f32,

    motor_stall_torque: f32,
    motor_max_speed: f32,
}

const COLLIDER_MARGIN: f32 = 0.001;

fn cuboid(l: f32, w: f32, h: f32) -> ColliderDesc<f32> {
    ColliderDesc::new(ShapeHandle::new(Cuboid::new(Vector3::new(
        l / 2.0,
        w / 2.0,
        h / 2.0,
    ))))
    .density(1.1)
    .margin(COLLIDER_MARGIN)
}

fn ball(r: f32) -> ColliderDesc<f32> {
    ColliderDesc::new(ShapeHandle::new(Ball::new(r)))
        .density(1.1)
        .margin(COLLIDER_MARGIN)
}

fn isometry_zero() -> Isometry3<f32> {
    Isometry3::from_parts(
        Translation3::from(Vector3::zeros()),
        UnitQuaternion::new(Vector3::zeros()),
    )
}

fn isometry_xyz(x: f32, y: f32, z: f32) -> Isometry3<f32> {
    Isometry3::from_parts(
        Translation3::from(Vector3::new(x, y, z)),
        UnitQuaternion::new(Vector3::zeros()),
    )
}

const WHEEL_DISPLACEMENT_Z: f32 = (CAR_LENGTH / 2.0) - (CAR_WHEEL_RADIUS + CAR_WHEEL_SPACE);
const WHEEL_DISPLACEMENT_X: f32 = CAR_WIDTH / 2.0;
const BODY_LENGTH: f32 = CAR_LENGTH;
const BODY_WIDTH: f32 = CAR_WIDTH - 2.0 * (CAR_WHEEL_RADIUS + CAR_WHEEL_SPACE);
const BODY_HEIGHT: f32 = CAR_WHEEL_RADIUS;
const GROUND_THICKNESS: f32 = 0.1;

fn wheel_joint() -> RevoluteJoint<f32> {
    let mut joint = RevoluteJoint::new(Vector3::x_axis(), 0.0);
    joint.enable_angular_motor();
    joint.set_desired_angular_motor_velocity(0.0);
    joint
}

const MOTOR_STALL_TORQUE: f32 = 0.4 / 3.0;
const MOTOR_MAX_RPM: f32 = 220.0 * 3.0;

impl SimulatedWorld {
    pub fn new() -> Self {
        let mut mechanical_world = DefaultMechanicalWorld::new(Vector3::new(0.0, -9.81, 0.0));
        let mut bodies = DefaultBodySet::new();
        let mut colliders: DefaultColliderSet<f32> = DefaultColliderSet::new();
        let joint_constraints = DefaultJointConstraintSet::new();

        let mut car_root_desc =
            MultibodyDesc::new(FreeJoint::new(isometry_xyz(0.0, 0.4, 0.0))).name("car".to_owned());

        car_root_desc
            .add_child(FixedJoint::new(isometry_zero()))
            .set_name("body".to_owned());
        car_root_desc
            .add_child(wheel_joint())
            .set_name("bl".to_owned())
            .set_parent_shift(Vector3::new(
                WHEEL_DISPLACEMENT_X,
                0.0,
                -WHEEL_DISPLACEMENT_Z,
            ));
        car_root_desc
            .add_child(wheel_joint())
            .set_name("br".to_owned())
            .set_parent_shift(Vector3::new(
                -WHEEL_DISPLACEMENT_X,
                0.0,
                -WHEEL_DISPLACEMENT_Z,
            ));
        car_root_desc
            .add_child(wheel_joint())
            .set_name("fl".to_owned())
            .set_parent_shift(Vector3::new(
                WHEEL_DISPLACEMENT_X,
                0.0,
                WHEEL_DISPLACEMENT_Z,
            ));
        car_root_desc
            .add_child(wheel_joint())
            .set_name("fr".to_owned())
            .set_parent_shift(Vector3::new(
                -WHEEL_DISPLACEMENT_X,
                0.0,
                WHEEL_DISPLACEMENT_Z,
            ));

        let car_multibody = car_root_desc.build();
        let car_part_id_body = car_multibody
            .links_with_name("body")
            .last()
            .unwrap()
            .1
            .link_id();
        let car_part_id_bl = car_multibody
            .links_with_name("bl")
            .last()
            .unwrap()
            .1
            .link_id();
        let car_part_id_br = car_multibody
            .links_with_name("br")
            .last()
            .unwrap()
            .1
            .link_id();
        let car_part_id_fl = car_multibody
            .links_with_name("fl")
            .last()
            .unwrap()
            .1
            .link_id();
        let car_part_id_fr = car_multibody
            .links_with_name("fr")
            .last()
            .unwrap()
            .1
            .link_id();
        let car_root = bodies.insert(car_multibody);

        colliders.insert(
            cuboid(BODY_WIDTH, BODY_HEIGHT, BODY_LENGTH)
                .build(BodyPartHandle(car_root, car_part_id_body)),
        );
        colliders.insert(ball(CAR_WHEEL_RADIUS).build(BodyPartHandle(car_root, car_part_id_bl)));
        colliders.insert(ball(CAR_WHEEL_RADIUS).build(BodyPartHandle(car_root, car_part_id_br)));
        colliders.insert(ball(CAR_WHEEL_RADIUS).build(BodyPartHandle(car_root, car_part_id_fl)));
        colliders.insert(ball(CAR_WHEEL_RADIUS).build(BodyPartHandle(car_root, car_part_id_fr)));

        let ground_shape = ShapeHandle::new(Cuboid::new(Vector3::new(6.0, GROUND_THICKNESS, 6.0)));
        let ground = bodies.insert(Ground::new());
        let ground_collider = ColliderDesc::new(ground_shape)
            .translation(Vector3::y() * -GROUND_THICKNESS)
            .build(BodyPartHandle(ground, 0));
        colliders.insert(ground_collider);

        mechanical_world.set_timestep(1.0 / 120.0);

        SimulatedWorld {
            mechanical_world,
            geometrical_world: DefaultGeometricalWorld::new(),
            bodies,
            colliders,
            joint_constraints,
            force_generators: DefaultForceGeneratorSet::new(),

            ground,
            ground_part_count: 1,

            car: car_root,
            car_part_id_body,
            car_part_id_bl,
            car_part_id_br,
            car_part_id_fl,
            car_part_id_fr,
            car_motor_power_bl: 0.0,
            car_motor_power_br: 0.0,
            car_motor_power_fl: 0.0,
            car_motor_power_fr: 0.0,
            motor_stall_torque: MOTOR_STALL_TORQUE,
            motor_max_speed: (360.0 * MOTOR_MAX_RPM / 60.0).to_radians(),
        }
    }

    pub fn car_body(&self) -> BodyPartHandle<DefaultBodyHandle> {
        BodyPartHandle(self.car, self.car_part_id_body)
    }
    pub fn car_wheel_bl(&self) -> BodyPartHandle<DefaultBodyHandle> {
        BodyPartHandle(self.car, self.car_part_id_bl)
    }
    pub fn car_wheel_br(&self) -> BodyPartHandle<DefaultBodyHandle> {
        BodyPartHandle(self.car, self.car_part_id_br)
    }
    pub fn car_wheel_fl(&self) -> BodyPartHandle<DefaultBodyHandle> {
        BodyPartHandle(self.car, self.car_part_id_fl)
    }
    pub fn car_wheel_fr(&self) -> BodyPartHandle<DefaultBodyHandle> {
        BodyPartHandle(self.car, self.car_part_id_fr)
    }

    pub fn body_position(&self) -> ISO {
        let car = self.bodies.get(self.car).unwrap();
        let body = car.part(self.car_part_id_body).unwrap();
        body.position()
    }

    fn wheel_rotation(&self, wheel_part_id: usize) -> f32 {
        let car = self.bodies.get(self.car).unwrap();
        let body = car.part(self.car_part_id_body).unwrap();
        let wheel = car.part(wheel_part_id).unwrap();
        let body_rot = body.position().rotation;
        let wheel_rot = wheel.position().rotation;
        let wheel_rot = body_rot.inverse() * wheel_rot;
        wheel_rot.rot_x()
    }
    pub fn wheel_rotation_bl(&self) -> f32 {
        self.wheel_rotation(self.car_part_id_bl)
    }
    pub fn wheel_rotation_br(&self) -> f32 {
        self.wheel_rotation(self.car_part_id_br)
    }
    pub fn wheel_rotation_fl(&self) -> f32 {
        self.wheel_rotation(self.car_part_id_fl)
    }
    pub fn wheel_rotation_fr(&self) -> f32 {
        self.wheel_rotation(self.car_part_id_fr)
    }

    fn wheel_velocity(&self, wheel_part_id: usize) -> f32 {
        let car = self.bodies.get(self.car).unwrap();
        let body = car.part(self.car_part_id_body).unwrap();
        let wheel = car.part(wheel_part_id).unwrap();
        let body_velocity = body.velocity().angular;
        let wheel_velocity = wheel.velocity().angular;
        let wheel_velocity = wheel_velocity - body_velocity;
        wheel_velocity.x
    }
    pub fn wheel_velocity_bl(&self) -> f32 {
        self.wheel_velocity(self.car_part_id_bl)
    }
    pub fn wheel_velocity_br(&self) -> f32 {
        self.wheel_velocity(self.car_part_id_br)
    }
    pub fn wheel_velocity_fl(&self) -> f32 {
        self.wheel_velocity(self.car_part_id_fl)
    }
    pub fn wheel_velocity_fr(&self) -> f32 {
        self.wheel_velocity(self.car_part_id_fr)
    }

    fn power_ratio(&self, velocity: f32) -> f32 {
        let reduction = velocity / self.motor_max_speed;
        let reduction = if reduction > 1.0 { 1.0 } else { reduction };
        1.0 - reduction
    }

    /*
    fn apply_wheel_power(&mut self, wheel_part_id: usize, power: f32) {
        let velocity = self.wheel_velocity(wheel_part_id);
        let (target_velocity, max_torque) = if power > 0.0 {
            (
                self.motor_max_speed * power,
                if velocity > 0.0 {
                    self.motor_stall_torque // * self.power_ratio(velocity)
                } else {
                    self.motor_stall_torque
                },
            )
        } else if power < 0.0 {
            (
                self.motor_max_speed * power,
                if velocity < 0.0 {
                    self.motor_stall_torque // * self.power_ratio(-velocity)
                } else {
                    self.motor_stall_torque
                },
            )
        } else {
            (0.0, self.motor_stall_torque)
        };
        let car = self.bodies.multibody_mut(self.car).unwrap();
        let link = car.link_mut(wheel_part_id).unwrap();
        let joint = link.joint_mut();
        let joint = joint.downcast_mut::<RevoluteJoint<f32>>().unwrap();
        joint.set_desired_angular_motor_velocity(target_velocity);
        joint.set_max_angular_motor_torque(max_torque);
    }
    */
    fn apply_wheel_power(&mut self, wheel_part_id: usize, power: f32) {
        let target_velocity = self.motor_max_speed * power;
        let car = self.bodies.multibody_mut(self.car).unwrap();
        let link = car.link_mut(wheel_part_id).unwrap();
        let joint = link.joint_mut();
        let joint = joint.downcast_mut::<RevoluteJoint<f32>>().unwrap();
        joint.set_desired_angular_motor_velocity(target_velocity);
        joint.set_max_angular_motor_torque(self.motor_stall_torque);
    }

    pub fn apply_power(&mut self) {
        self.apply_wheel_power(self.car_part_id_bl, self.car_motor_power_bl);
        self.apply_wheel_power(self.car_part_id_br, self.car_motor_power_br);
        self.apply_wheel_power(self.car_part_id_fl, self.car_motor_power_fl);
        self.apply_wheel_power(self.car_part_id_fr, self.car_motor_power_fr);
    }

    pub fn set_motor_power(&mut self, bl: f32, br: f32, fl: f32, fr: f32) {
        self.car_motor_power_bl = bl;
        self.car_motor_power_br = br;
        self.car_motor_power_fl = fl;
        self.car_motor_power_fr = fr;
    }

    fn next_ground_part_count(&mut self) -> usize {
        self.ground_part_count += 1;
        self.ground_part_count
    }

    fn add_map_box(&mut self, section_box: &map::MapSectionBox) {
        let translation = Vector3::new(
            section_box.center.x,
            section_box.center.y,
            section_box.center.z,
        );
        let rotation = section_box.rotation;
        let mut box_collider_desc =
            cuboid(section_box.width, section_box.height, section_box.length)
                .translation(translation);
        if let Some(axis) = rotation.axis() {
            box_collider_desc =
                box_collider_desc.rotation(rotation.angle() * NaV3::new(axis.x, axis.y, axis.z));
        }
        let box_collider =
            box_collider_desc.build(BodyPartHandle(self.ground, self.next_ground_part_count()));
        self.colliders.insert(box_collider);
    }

    pub fn setup_map(&mut self, map: &Map) {
        let segments = map_segmentation(map);
        for segment in segments.iter() {
            self.add_map_box(&segment.floor_box());
            self.add_map_box(&segment.left_box());
            self.add_map_box(&segment.right_box());
        }
    }

    pub fn step(&mut self) {
        self.apply_power();
        self.mechanical_world.step(
            &mut self.geometrical_world,
            &mut self.bodies,
            &mut self.colliders,
            &mut self.joint_constraints,
            &mut self.force_generators,
        );
    }

    pub fn run_testbed(self) {
        use nphysics_testbed3d::Testbed;

        let mut testbed = Testbed::new_empty();
        // Tell the testbed your ground has an handle equal to `ground_handle`.
        // This will make it be drawn in gray.
        testbed.set_ground_handle(Some(self.ground));
        // Provide to the testbed all the components of our physics simulation.
        testbed.set_world(
            self.mechanical_world,
            self.geometrical_world,
            self.bodies,
            self.colliders,
            self.joint_constraints,
            self.force_generators,
        );
        // Adjust the initial camera pose.
        testbed.look_at(Point3::new(0.0, 1.0, -1.0), Point3::new(0.0, 0.0, 0.0));

        testbed.run();
    }
}
