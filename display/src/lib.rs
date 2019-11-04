use kiss3d::camera::camera::Camera;
use kiss3d::camera::ArcBall;
use kiss3d::light::Light;
use kiss3d::scene::SceneNode;
use kiss3d::window::Window;
use nalgebra::RealField;
use nalgebra::{Point3, Translation3, Vector3};

use map::*;
use protocol::map::Map;

mod ui;
use ui::{gui, Ids, UiState};

pub struct VisualizedWorld {
    window: Window,
    camera: ArcBall,

    car: SceneNode,
    car_body: SceneNode,
    car_lasers: SceneNode,
    car_wheel_bl_axle: SceneNode,
    car_wheel_br_axle: SceneNode,
    car_wheel_fl_axle: SceneNode,
    car_wheel_fr_axle: SceneNode,

    ground: SceneNode,
    track: Vec<SceneNode>,

    ids: Ids,
    ui_state: UiState,
}

impl VisualizedWorld {
    pub fn new(car: &Car) -> Self {
        let mut window = Window::new("Folks Race Simulation");
        window.set_light(Light::StickToCamera);

        let camera = ArcBall::new(Point3::new(0.0, 2.0, -2.0), Point3::new(0.0, 0.0, 0.0));

        let mut car_group = window.add_group();

        let mut car_body = car_group.add_cube(car.body_w(), car.body_h(), car.body_l());
        car_body.set_color(0.0, 0.0, 1.0);
        car_body.set_local_translation(Translation3::from(car.body_position()));
        let mut car_lasers = car_group.add_sphere(0.025);
        car_lasers.set_color(1.0, 1.0, 0.0);
        car_lasers.set_local_translation(Translation3::from(car.laser_position()));

        let mut car_wheel_bl_joint = car_group.add_group();
        let mut car_wheel_br_joint = car_group.add_group();
        let mut car_wheel_fl_joint = car_group.add_group();
        let mut car_wheel_fr_joint = car_group.add_group();
        for joint in [
            &mut car_wheel_bl_joint,
            &mut car_wheel_br_joint,
            &mut car_wheel_fl_joint,
            &mut car_wheel_fr_joint,
        ]
        .iter_mut()
        {
            joint.set_local_rotation(NaQ::from_axis_angle(&NaV3::z_axis(), -f32::frac_pi_2()));
        }

        let mut car_wheel_bl_axle = car_wheel_bl_joint.add_group();
        let mut car_wheel_br_axle = car_wheel_br_joint.add_group();
        let mut car_wheel_fl_axle = car_wheel_fl_joint.add_group();
        let mut car_wheel_fr_axle = car_wheel_fr_joint.add_group();

        for axle in [
            &mut car_wheel_bl_axle,
            &mut car_wheel_br_axle,
            &mut car_wheel_fl_axle,
            &mut car_wheel_fr_axle,
        ]
        .iter_mut()
        {
            let mut wheel = axle.add_cylinder(car.wheel_radius, car.wheel_thickness);
            let mut rim1 = axle.add_quad(
                (car.wheel_radius + CAR_WHEEL_SPACE) * 2.0,
                car.wheel_thickness + CAR_WHEEL_SPACE,
                1,
                1,
            );
            let mut rim2 = axle.add_quad(
                (car.wheel_radius + CAR_WHEEL_SPACE) * 2.0,
                car.wheel_thickness + CAR_WHEEL_SPACE,
                1,
                1,
            );
            let mut rim3 = axle.add_quad(
                (car.wheel_radius + CAR_WHEEL_SPACE) * 2.0,
                car.wheel_thickness + CAR_WHEEL_SPACE,
                1,
                1,
            );
            wheel.set_color(0.2, 0.2, 0.2);
            rim1.set_color(0.0, 0.8, 0.0);
            rim2.set_color(0.0, 0.8, 0.0);
            rim3.set_color(0.0, 0.8, 0.0);
            rim2.set_local_rotation(NaQ::from_axis_angle(&NaV3::y_axis(), f32::frac_pi_3()));
            rim3.set_local_rotation(NaQ::from_axis_angle(
                &NaV3::y_axis(),
                f32::frac_pi_3() * 2.0,
            ));
        }

        car_wheel_bl_joint.set_local_translation(Translation3::from(Vector3::new(
            car.wheel_x(),
            car.wheel_y(),
            -car.wheel_z(),
        )));
        car_wheel_br_joint.set_local_translation(Translation3::from(Vector3::new(
            -car.wheel_x(),
            car.wheel_y(),
            -car.wheel_z(),
        )));
        car_wheel_fl_joint.set_local_translation(Translation3::from(Vector3::new(
            car.wheel_x(),
            car.wheel_y(),
            car.wheel_z(),
        )));
        car_wheel_fr_joint.set_local_translation(Translation3::from(Vector3::new(
            -car.wheel_x(),
            car.wheel_y(),
            car.wheel_z(),
        )));

        let mut ground = window.add_quad(6.0, 6.0, 1, 1);
        ground.set_color(0.0, 1.0, 0.0);
        ground.set_local_rotation(NaQ::from_axis_angle(&NaV3::x_axis(), f32::frac_pi_2()));

        let track = vec![];

        let mut gen = window.conrod_ui_mut().widget_id_generator();
        let ui_state = UiState::new(&mut gen);
        let ids = Ids::new(gen);

        VisualizedWorld {
            window,
            camera,
            car: car_group,
            car_body,
            car_lasers,
            car_wheel_bl_axle,
            car_wheel_br_axle,
            car_wheel_fl_axle,
            car_wheel_fr_axle,
            ground,
            track,
            ids,
            ui_state,
        }
    }

    fn add_map_box(&mut self, section_box: &MapSectionBox, light: bool) {
        let mut cube =
            self.window
                .add_cube(section_box.width, section_box.height, section_box.length);
        cube.set_local_rotation(section_box.rotation);
        cube.set_local_translation(Translation3::from(section_box.center));
        if light {
            cube.set_color(0.6, 0.6, 0.6);
        } else {
            cube.set_color(0.4, 0.4, 0.4);
        }
    }
    pub fn setup_map(&mut self, map: &Map) {
        self.track.clear();
        let segments = map_segmentation(map);
        for segment in segments.iter() {
            self.add_map_box(&segment.floor_box(), false);
            self.add_map_box(&segment.left_box(), segment.is_lighter);
            self.add_map_box(&segment.right_box(), segment.is_lighter);
        }
    }

    pub fn set_car_position(&mut self, pos: Vector3<f32>) {
        self.car.set_local_translation(Translation3::from(pos));
    }

    pub fn set_car_rotation(&mut self, rotation: NaQ) {
        self.car.set_local_rotation(rotation);
    }

    pub fn set_wheel_angles(&mut self, bl: f32, br: f32, fl: f32, fr: f32) {
        self.car_wheel_bl_axle
            .set_local_rotation(NaQ::from_axis_angle(&NaV3::y_axis(), bl));
        self.car_wheel_br_axle
            .set_local_rotation(NaQ::from_axis_angle(&NaV3::y_axis(), br));
        self.car_wheel_fl_axle
            .set_local_rotation(NaQ::from_axis_angle(&NaV3::y_axis(), fl));
        self.car_wheel_fr_axle
            .set_local_rotation(NaQ::from_axis_angle(&NaV3::y_axis(), fr));
    }

    fn update_camera(&mut self) {
        let c = &self.ui_state.camera;
        if c.is_follow() {
            let to = self
                .car_body
                .data()
                .world_transformation()
                .translation
                .transform_point(&Point3::origin());
            let heading = self.car_body.data().world_transformation().rotation.rot_y();
            let to = if c.follow_strife != 0.0 {
                let right_heading = heading + f32::frac_pi_2();
                let right_rotation = NaQ::from_axis_angle(&NaV3::y_axis(), right_heading);
                let right_direction = right_rotation.transform_vector(&NaV3::new(0.0, 0.0, 1.0));
                to + (right_direction * c.follow_strife)
            } else {
                to
            };
            let heading = heading + c.follow_heading.to_radians();
            let heading_rotation = NaQ::from_axis_angle(&NaV3::y_axis(), heading);
            let camera_distance = c.follow_distance;
            let camera_height = camera_distance * c.follow_pitch.to_radians().sin();
            let relative_eye_flat_distance = camera_distance * c.follow_pitch.to_radians().cos();
            let relative_eye = heading_rotation.transform_vector(&NaV3::new(
                0.0,
                camera_height,
                -relative_eye_flat_distance,
            ));
            let eye = to + relative_eye;
            self.camera.look_at(eye, to);
        } else {
            let to = Point3::new(-c.target_x, c.target_y, c.target_z);
            let eye_rotation =
                NaQ::from_axis_angle(&NaV3::y_axis(), (c.eye_heading + 0.0).to_radians());
            let eye_distance = c.eye_distance;
            let eye_height = c.eye_distance * c.eye_pitch.to_radians().sin();
            let relative_eye_flat_distance = eye_distance * c.eye_pitch.to_radians().cos();
            let relative_eye = eye_rotation.transform_vector(&NaV3::new(
                0.0,
                eye_height,
                -relative_eye_flat_distance,
            ));
            let eye = to + relative_eye;
            self.camera.look_at(eye, to);
        }
    }

    pub fn render(&mut self) -> bool {
        self.update_camera();
        let result = self.window.render_with_camera(&mut self.camera);
        self.ui_state.window_width = self.window.width().into();
        self.ui_state.window_height = self.window.height().into();
        if result {
            let mut ui = self.window.conrod_ui_mut().set_widgets();
            gui(&mut ui, &self.ids, &mut self.ui_state);
        }
        result
    }

    pub fn ui(&self) -> &UiState {
        &self.ui_state
    }
}
