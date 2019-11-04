use cnrd::color::Color;
use cnrd::color::Colorable;
use cnrd::position::Positionable;
use cnrd::position::Sizeable;
use cnrd::widget::button::{Button, Flat, TimesClicked};
use cnrd::widget::canvas::Canvas;
use cnrd::widget::id::{Generator, Id};
use cnrd::widget::primitive::text::Text;
use cnrd::widget::scrollbar::Scrollbar;
use cnrd::widget::xy_pad::XYPad;
use cnrd::widget_ids;
use cnrd::Borderable;
use cnrd::Labelable;
use cnrd::Widget;
use kiss3d::conrod as cnrd;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum UiActivity {
    Idle,
    Commands,
    Camera,
    Manual,
}

type DIM = cnrd::position::Scalar;

#[derive(Clone, Copy)]
pub struct DirectPower {
    pub front: f32,
    pub side: f32,
}

impl DirectPower {
    pub fn none() -> Option<Self> {
        None
    }
    pub fn from_front_side(front: f32, side: f32) -> Option<Self> {
        Some(Self { front, side })
    }
    pub fn power(&self) -> (f32, f32, f32, f32) {
        (
            self.front + self.side,
            self.front - self.side,
            self.front + self.side,
            self.front - self.side,
        )
    }
}

const CAMERA_EYE_DISTANCE: f32 = 4.0;
const CAMERA_EYE_PITCH: f32 = 89.9;
const CAMERA_EYE_HEADING: f32 = 0.0;
const CAMERA_FOLLOW_DISTANCE: f32 = 0.5;
const CAMERA_FOLLOW_PITCH: f32 = 30.0;
const CAMERA_FOLLOW_HEADING: f32 = 0.0;

const CAMERA_TARGET_X_MIN: f32 = -2.5;
const CAMERA_TARGET_X_MAX: f32 = 2.5;
const CAMERA_TARGET_Y_MIN: f32 = -2.5;
const CAMERA_TARGET_Y_MAX: f32 = 2.5;
const CAMERA_EYE_PITCH_MIN: f32 = 0.1;
const CAMERA_EYE_PITCH_MAX: f32 = 89.9;
const CAMERA_EYE_HEADING_MIN: f32 = -180.0;
const CAMERA_EYE_HEADING_MAX: f32 = 180.0;

const CAMERA_FOLLOW_DISTANCE_MIN: f32 = 0.25;
const CAMERA_FOLLOW_DISTANCE_MAX: f32 = 2.0;
const CAMERA_FOLLOW_STRIFE_MIN: f32 = -1.0;
const CAMERA_FOLLOW_STRIFE_MAX: f32 = 1.0;
const CAMERA_FOLLOW_PITCH_MIN: f32 = 0.0;
const CAMERA_FOLLOW_PITCH_MAX: f32 = 89.9;
const CAMERA_FOLLOW_HEADING_MIN: f32 = -180.0;
const CAMERA_FOLLOW_HEADING_MAX: f32 = 180.0;

pub struct CameraState {
    follow: bool,
    pub target_x: f32,
    pub target_y: f32,
    pub target_z: f32,
    pub eye_distance: f32,
    pub eye_pitch: f32,
    pub eye_heading: f32,
    pub follow_distance: f32,
    pub follow_strife: f32,
    pub follow_pitch: f32,
    pub follow_heading: f32,
}

impl CameraState {
    pub fn new() -> Self {
        Self {
            follow: false,
            target_x: 0.0,
            target_y: 0.0,
            target_z: 0.0,
            eye_distance: CAMERA_EYE_DISTANCE,
            eye_pitch: CAMERA_EYE_PITCH,
            eye_heading: CAMERA_EYE_HEADING,
            follow_distance: CAMERA_FOLLOW_DISTANCE,
            follow_strife: 0.0,
            follow_pitch: CAMERA_FOLLOW_PITCH,
            follow_heading: CAMERA_FOLLOW_HEADING,
        }
    }

    pub fn is_fixed(&self) -> bool {
        !self.follow
    }
    pub fn set_fixed(&mut self) {
        self.follow = false;
    }
    pub fn reset_fixed(&mut self) {
        self.target_x = 0.0;
        self.target_y = 0.0;
        self.target_z = 0.0;
        self.eye_distance = CAMERA_EYE_DISTANCE;
        self.eye_pitch = CAMERA_EYE_PITCH;
        self.eye_heading = CAMERA_EYE_HEADING;
    }

    pub fn is_follow(&self) -> bool {
        self.follow
    }
    pub fn set_follow(&mut self) {
        self.follow = true;
    }
    pub fn reset_follow(&mut self) {
        self.follow_distance = CAMERA_FOLLOW_DISTANCE;
        self.follow_strife = 0.0;
        self.follow_pitch = CAMERA_FOLLOW_PITCH;
        self.follow_heading = CAMERA_FOLLOW_HEADING;
    }
}

#[derive(Clone)]
pub struct LogLine {
    pub id: Id,
    pub line: String,
}

pub struct UiState {
    pub activity: UiActivity,
    pub log: LogLines,
    pub window_width: DIM,
    pub window_height: DIM,
    pub power: Option<DirectPower>,
    pub camera: CameraState,
}

const BASE_MARGIN: DIM = 5.0;
const BUTTON_W_SCALE: DIM = 0.15;
const BUTTON_H_SCALE: DIM = 0.1;
const JOYSTICK_SCALE: DIM = 0.25;
const JOYSTICK_LINE_THICKNESS: DIM = 3.0;
const LOG_W_SCALE: DIM = 0.6;
const LOG_H_SCALE: DIM = 0.25;
const SCROLL_W_SCALE: DIM = 0.1;
const COMMAND_DISPLACEMENT_SCALE: DIM = 4.2;
const MENU_TEXT_SIZE_SCALE: f64 = 0.025;
const LOG_TEXT_SIZE_SCALE: f64 = 0.020;
const BUTTON_NORMAL_COLOR: Color = cnrd::color::LIGHT_GREY;
const BUTTON_CANCEL_COLOR: Color = cnrd::color::RED;
const BUTTON_OK_COLOR: Color = cnrd::color::GREEN;
const TRANSPARENT_COLOR: Color = cnrd::color::TRANSPARENT;
const LOG_LENGTH: usize = 30;

impl UiState {
    pub fn new(gen: &mut Generator) -> Self {
        let mut s = Self {
            activity: UiActivity::Idle,
            log: LogLines::new(gen),
            window_width: 640.0,
            window_height: 480.0,
            power: DirectPower::none(),
            camera: CameraState::new(),
        };
        for i in 1..15 {
            s.log.append(&format!("Line {}", i));
        }
        s
    }

    pub fn button_w(&self) -> DIM {
        self.window_width * BUTTON_W_SCALE
    }
    pub fn button_h(&self) -> DIM {
        self.window_height * BUTTON_H_SCALE
    }
    pub fn joystick_w(&self) -> DIM {
        self.window_width * JOYSTICK_SCALE
    }
    pub fn joystick_h(&self) -> DIM {
        self.window_width * JOYSTICK_SCALE
    }
    pub fn log_w(&self) -> DIM {
        self.window_width * LOG_W_SCALE
    }
    pub fn log_h(&self) -> DIM {
        self.window_height * LOG_H_SCALE
    }
    pub fn scroll_w(&self) -> DIM {
        self.window_width * SCROLL_W_SCALE
    }

    pub fn command_displacement(&self, c: Command) -> DIM {
        let index = c.index() as DIM;
        let max = (Command::len() - 1) as DIM;
        let dim = 0.5 - (index / max);
        self.button_h() * COMMAND_DISPLACEMENT_SCALE * dim
    }

    pub fn menu_text_size(&self) -> cnrd::FontSize {
        (self.window_height * MENU_TEXT_SIZE_SCALE) as u32
    }
    pub fn log_text_size(&self) -> cnrd::FontSize {
        (self.window_height * LOG_TEXT_SIZE_SCALE) as u32
    }

    pub fn append_log(&mut self, line: &str) {
        self.log.append(line);
    }
}

widget_ids! {
    pub struct Ids {
        // The main container (canvas)
        base,
        // Button at the top left of the screen
        button_left,
        // Button at the top center of the screen
        button_center,
        // Button at the top right of the screen
        button_right,
        // The message log at the bottom (canvas)
        log,
        // The message log scrollbar
        log_scrollbar,
        // Command button (reset)
        commands_reset,
        // Command button (start)
        commands_start,
        // Command button (stop)
        commands_stop,
        // Command button (restart)
        commands_restart,
        // Command button (clear_log)
        commands_clear_log,
        // Left manual controls
        joystick_left,
        // Right manual controls
        joystick_right,
    }
}

pub struct LogLines {
    ids: [Id; LOG_LENGTH],
    lines: [String; LOG_LENGTH],
    start: usize,
    end: usize,
    capacity: usize,
}

impl LogLines {
    pub fn new(gen: &mut Generator) -> Self {
        let mut result = Self {
            ids: Default::default(),
            lines: Default::default(),
            start: 0,
            end: 0,
            capacity: LOG_LENGTH,
        };
        for i in 0..LOG_LENGTH {
            result.ids[i] = gen.next();
        }
        result
    }

    pub fn count(&self) -> usize {
        LOG_LENGTH - self.capacity
    }

    pub fn line_at(&self, index: usize) -> &str {
        &self.lines[(self.start + index) % LOG_LENGTH]
    }
    pub fn id_at(&self, index: usize) -> Id {
        self.ids[(self.start + index) % LOG_LENGTH]
    }

    fn next(&self, index: usize) -> usize {
        (index + 1) % LOG_LENGTH
    }

    pub fn append(&mut self, line: &str) {
        if self.capacity == 0 {
            self.start = self.next(self.start);
            self.capacity += 1;
        };
        self.lines[self.end] = String::from(line);
        self.end = self.next(self.end);
        self.capacity -= 1;
    }
}

fn top_button<'a>(
    label: &'a str,
    color: Color,
    ids: &Ids,
    state: &mut UiState,
) -> Button<'a, Flat> {
    Button::new()
        .label(label)
        .label_font_size(state.menu_text_size())
        .color(color)
        .parent(ids.base)
        .w_h(state.button_w(), state.button_h())
}

fn top_left_button(
    label: &str,
    color: Color,
    ui: &mut cnrd::UiCell,
    ids: &Ids,
    state: &mut UiState,
) -> TimesClicked {
    top_button(label, color, ids, state)
        .top_left_with_margin_on(ids.base, 0.0)
        .set(ids.button_left, ui)
}

fn top_center_button(
    label: &str,
    color: Color,
    ui: &mut cnrd::UiCell,
    ids: &Ids,
    state: &mut UiState,
) -> TimesClicked {
    top_button(label, color, ids, state)
        .mid_top_with_margin_on(ids.base, 0.0)
        .set(ids.button_center, ui)
}

fn top_right_button(
    label: &str,
    color: Color,
    ui: &mut cnrd::UiCell,
    ids: &Ids,
    state: &mut UiState,
) -> TimesClicked {
    top_button(label, color, ids, state)
        .top_right_with_margin_on(ids.base, 0.0)
        .set(ids.button_right, ui)
}

static COMMANDS: [&str; 5] = ["RESET", "START", "STOP", "RESTART", "CLEAR LOG"];
#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(usize)]
pub enum Command {
    Reset = 0,
    Start = 1,
    Stop = 2,
    Restart = 3,
    ClearLog = 4,
}

impl Command {
    pub fn len() -> usize {
        COMMANDS.len()
    }
    pub fn from(index: usize) -> Self {
        match index {
            0 => Command::Reset,
            1 => Command::Start,
            2 => Command::Stop,
            3 => Command::Restart,
            4 => Command::ClearLog,
            _ => panic!(format!("Invalid command index {}", index)),
        }
    }
    pub fn index(&self) -> usize {
        *self as usize
    }
    pub fn text(&self) -> &'static str {
        COMMANDS[self.index()]
    }
    pub fn id(&self, ids: &Ids) -> Id {
        match self {
            Command::Reset => ids.commands_reset,
            Command::Start => ids.commands_start,
            Command::Stop => ids.commands_stop,
            Command::Restart => ids.commands_restart,
            Command::ClearLog => ids.commands_clear_log,
        }
    }
}

pub fn gui(ui: &mut cnrd::UiCell, ids: &Ids, state: &mut UiState) {
    Canvas::new()
        .pad(BASE_MARGIN)
        .length_weight(1.0)
        .color(TRANSPARENT_COLOR)
        .set(ids.base, ui);

    if state.activity == UiActivity::Camera {
        for _ in top_left_button(
            if state.camera.is_follow() {
                "OK"
            } else {
                "FOLLOW"
            },
            if state.camera.is_follow() {
                BUTTON_OK_COLOR
            } else {
                BUTTON_NORMAL_COLOR
            },
            ui,
            ids,
            state,
        ) {
            if state.camera.is_follow() {
                state.activity = UiActivity::Idle;
            } else {
                state.camera.set_follow();
            }
        }
        for _ in top_center_button("RESET", BUTTON_NORMAL_COLOR, ui, ids, state) {
            if state.camera.follow {
                state.camera.reset_follow();
            } else {
                state.camera.reset_fixed();
            }
        }
        for _ in top_right_button(
            if state.camera.is_fixed() {
                "OK"
            } else {
                "FIXED"
            },
            if state.camera.is_fixed() {
                BUTTON_OK_COLOR
            } else {
                BUTTON_NORMAL_COLOR
            },
            ui,
            ids,
            state,
        ) {
            if state.camera.is_fixed() {
                state.activity = UiActivity::Idle;
            } else {
                state.camera.set_fixed();
            }
        }
    } else {
        if state.activity == UiActivity::Commands || state.activity == UiActivity::Idle {
            for _ in top_left_button(
                if state.activity == UiActivity::Commands {
                    "CANCEL"
                } else {
                    "COMMANDS"
                },
                if state.activity == UiActivity::Commands {
                    BUTTON_CANCEL_COLOR
                } else {
                    BUTTON_NORMAL_COLOR
                },
                ui,
                ids,
                state,
            ) {
                if state.activity == UiActivity::Commands {
                    state.activity = UiActivity::Idle;
                } else {
                    state.activity = UiActivity::Commands;
                }
            }
        }
        if state.activity == UiActivity::Idle {
            for _ in top_center_button("CAMERA", BUTTON_NORMAL_COLOR, ui, ids, state) {
                state.activity = UiActivity::Camera;
            }
        }
        if state.activity == UiActivity::Manual || state.activity == UiActivity::Idle {
            for _ in top_right_button(
                if state.activity == UiActivity::Manual {
                    "CANCEL"
                } else {
                    "MANUAL"
                },
                if state.activity == UiActivity::Manual {
                    BUTTON_CANCEL_COLOR
                } else {
                    BUTTON_NORMAL_COLOR
                },
                ui,
                ids,
                state,
            ) {
                state.power = DirectPower::none();
                if state.activity == UiActivity::Manual {
                    state.activity = UiActivity::Idle;
                } else {
                    state.activity = UiActivity::Manual;
                }
            }
        }
    }

    Canvas::new()
        .parent(ids.base)
        .mid_bottom()
        .w_h(state.log_w(), state.log_h())
        .scroll_kids_vertically()
        .color(TRANSPARENT_COLOR)
        .border_color(TRANSPARENT_COLOR)
        .set(ids.log, ui);
    Scrollbar::y_axis(ids.log)
        .auto_hide(true)
        .w(state.scroll_w())
        .set(ids.log_scrollbar, ui);

    if state.log.count() > 0 {
        Text::new(state.log.line_at(0))
            .parent(ids.log)
            .mid_top()
            .font_size(state.log_text_size())
            .set(state.log.id_at(0), ui);
        for i in 1..state.log.count() {
            let previous = state.log.id_at(i - 1);
            Text::new(state.log.line_at(i))
                .parent(ids.log)
                .down_from(previous, 1.0)
                .font_size(state.log_text_size())
                .set(state.log.id_at(i), ui);
        }
    }

    /*
    println!(
        "Activity: {} (camera follow {})",
        match state.activity {
            UiActivity::Idle => "Base",
            UiActivity::Camera => "Camera",
            UiActivity::Commands => "Commands",
            UiActivity::Manual => "Manual",
        },
        state.camera.follow
    );
    */

    match state.activity {
        UiActivity::Idle => {}
        UiActivity::Commands => {
            let mut command: Option<Command> = None;
            for i in 0..Command::len() {
                let c = Command::from(i);
                for _ in Button::new()
                    .label(c.text())
                    .label_font_size(state.menu_text_size())
                    .color(BUTTON_NORMAL_COLOR)
                    .parent(ids.base)
                    .w_h(state.button_w(), state.button_h())
                    .x_y_relative_to(ids.base, 0.0, state.command_displacement(c))
                    .set(c.id(ids), ui)
                {
                    command = Some(c);
                }
            }
            if let Some(c) = command {
                println!("Command {}", c.text());
                state.activity = UiActivity::Idle;
            }
        }
        UiActivity::Camera => {
            if state.camera.is_fixed() {
                for (x, y) in XYPad::new(
                    state.camera.target_x,
                    CAMERA_TARGET_X_MIN,
                    CAMERA_TARGET_X_MAX,
                    state.camera.target_y,
                    CAMERA_TARGET_Y_MIN,
                    CAMERA_TARGET_Y_MAX,
                )
                .color(TRANSPARENT_COLOR)
                .line_thickness(JOYSTICK_LINE_THICKNESS)
                .w_h(state.joystick_w(), state.joystick_h())
                .parent(ids.base)
                .mid_left_of(ids.base)
                .set(ids.joystick_left, ui)
                {
                    state.camera.target_x = x;
                    state.camera.target_y = 0.0;
                    state.camera.target_z = y;
                }
                for (x, y) in XYPad::new(
                    state.camera.eye_heading,
                    CAMERA_EYE_HEADING_MIN,
                    CAMERA_EYE_HEADING_MAX,
                    state.camera.eye_pitch,
                    CAMERA_EYE_PITCH_MIN,
                    CAMERA_EYE_PITCH_MAX,
                )
                .color(TRANSPARENT_COLOR)
                .line_thickness(JOYSTICK_LINE_THICKNESS)
                .w_h(state.joystick_w(), state.joystick_h())
                .parent(ids.base)
                .mid_right_of(ids.base)
                .set(ids.joystick_right, ui)
                {
                    state.camera.eye_heading = x;
                    state.camera.eye_pitch = y;
                    state.camera.eye_distance = CAMERA_EYE_DISTANCE;
                }
            } else {
                for (x, y) in XYPad::new(
                    state.camera.follow_strife,
                    CAMERA_FOLLOW_STRIFE_MIN,
                    CAMERA_FOLLOW_STRIFE_MAX,
                    state.camera.follow_distance,
                    CAMERA_FOLLOW_DISTANCE_MIN,
                    CAMERA_FOLLOW_DISTANCE_MAX,
                )
                .color(TRANSPARENT_COLOR)
                .line_thickness(JOYSTICK_LINE_THICKNESS)
                .w_h(state.joystick_w(), state.joystick_h())
                .parent(ids.base)
                .mid_left_of(ids.base)
                .set(ids.joystick_left, ui)
                {
                    state.camera.follow_strife = x;
                    state.camera.follow_distance = y;
                }
                for (x, y) in XYPad::new(
                    state.camera.follow_heading,
                    CAMERA_FOLLOW_HEADING_MIN,
                    CAMERA_FOLLOW_HEADING_MAX,
                    state.camera.follow_pitch,
                    CAMERA_FOLLOW_PITCH_MIN,
                    CAMERA_FOLLOW_PITCH_MAX,
                )
                .color(TRANSPARENT_COLOR)
                .line_thickness(JOYSTICK_LINE_THICKNESS)
                .w_h(state.joystick_w(), state.joystick_h())
                .parent(ids.base)
                .mid_right_of(ids.base)
                .set(ids.joystick_right, ui)
                {
                    state.camera.follow_heading = x;
                    state.camera.follow_pitch = y;
                }
            }
        }
        UiActivity::Manual => {
            if let None = state.power {
                state.power = DirectPower::from_front_side(0.0, 0.0);
            }
            for (side, front) in XYPad::new(
                state.power.unwrap().side,
                -1.0,
                1.0,
                state.power.unwrap().front,
                -1.0,
                1.0,
            )
            .color(TRANSPARENT_COLOR)
            .line_thickness(JOYSTICK_LINE_THICKNESS)
            .w_h(state.joystick_w(), state.joystick_h())
            .parent(ids.base)
            .mid_right_of(ids.base)
            .set(ids.joystick_right, ui)
            {
                state.power = DirectPower::from_front_side(front, side);
            }
        }
    }
}
