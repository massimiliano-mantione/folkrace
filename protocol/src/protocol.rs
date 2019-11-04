use hal::{LASER_COUNT,ProtocolBuffer};

pub const MAX_LOG_LINE_SIZE: usize = 200;

const CODE_MINUS: u8 = '-' as u8;
const CODE_SEPARATOR: u8 = ':' as u8;
const CODE_END: u8 = '\n' as u8;

fn append_code(buf: &mut ProtocolBuffer, index: usize, code: u8) -> usize {
    buf[index] = code;
    index + 1
}

fn append_separator(buf: &mut ProtocolBuffer, index: usize) -> usize {
    append_code(buf, index, CODE_SEPARATOR)
}

fn append_end(buf: &mut ProtocolBuffer, index: usize) -> usize {
    append_code(buf, index, CODE_END)
}

/// Ok is next index, Err is index of wrong character
fn match_code(buf: &ProtocolBuffer, index: usize, code: u8) -> Result<usize, usize> {
    if buf[index] == code {
        Ok(index + 1)
    } else {
        Err(index)
    }
}

fn match_separator(buf: &ProtocolBuffer, index: usize) -> Result<usize, usize> {
    match_code(buf, index, CODE_SEPARATOR)
}

fn match_end(buf: &ProtocolBuffer, index: usize) -> Result<usize, usize> {
    match_code(buf, index, CODE_END)
}

fn write_string(buf: &mut ProtocolBuffer, index: usize, s: &str) -> usize {
    let mut index = index;
    for c in s.chars() {
        buf[index] = c as u8;
        index += 1;
    }
    index
}

/// Ok is next index, Err is index of wrong character
fn match_string(buf: &ProtocolBuffer, index: usize, s: &str) -> Result<usize, usize> {
    let mut index = index;
    for c in s.chars() {
        if buf[index] != c as u8 {
            return Err(index);
        }
        index += 1;
    }
    Ok(index)
}

fn digit_value(code: u8) -> Option<i32> {
    if code >= '0' as u8 && code <= '9' as u8 {
        Some(code as i32 - '0' as i32)
    } else {
        None
    }
}

fn digit_code(digit: i32) -> u8 {
    digit as u8 + '0' as u8
}

fn write_i32(buf: &mut ProtocolBuffer, index: usize, value: i32) -> usize {
    let mut value = value;
    let mut index = index;
    if value == 0 {
        buf[index] = digit_code(0);
        index += 1;
    } else {
        if value < 0 {
            buf[index] = CODE_MINUS;
            value = -value;
            index += 1;
        }

        let mut pow10 = 10;
        while pow10 <= value {
            pow10 *= 10
        }
        pow10 /= 10;

        while pow10 > 0 {
            let digit = value / pow10;
            value = value % pow10;
            pow10 /= 10;
            buf[index] = digit_code(digit);
            index += 1;
        }
    }
    index
}

/// Ok is value and next index, Err is index of wrong character
fn match_i32(buf: &ProtocolBuffer, index: usize) -> Result<(i32, usize), usize> {
    let mut index = index;
    let negative = buf[index] == CODE_MINUS;
    if negative {
        index += 1;
    }
    match digit_value(buf[index]) {
        None => Err(index),
        Some(v) => {
            let mut value = v;
            index += 1;
            loop {
                match digit_value(buf[index]) {
                    Some(v) => {
                        value *= 10;
                        value += v;
                        index += 1;
                    }
                    None => break,
                }
            }
            Ok((if negative { -value } else { value }, index))
        }
    }
}

/// Motor power (from -100 to +100)
pub type ProtocolMotorPower = i32;

#[derive(Clone, Copy, PartialEq, Eq)]
/// Motors power data
pub struct MotorsPowerData {
    pub back_left: ProtocolMotorPower,
    pub back_right: ProtocolMotorPower,
    pub front_left: ProtocolMotorPower,
    pub front_right: ProtocolMotorPower,
}

/// Length of track item in mm
pub type ProtocolLinearDimension = i32;

/// Acceleration in mm/s2
pub type ProtocolLinearAcceleration = i32;

/// Angle in deg, from -360 to +360, positive is clockwise
pub type ProtocolAngle = i32;

#[derive(Clone, Copy, PartialEq, Eq)]
/// Description of straight map section
pub struct ProtocolMapSectionDataStraight {
    // Starting width
    pub width_start: ProtocolLinearDimension,
    // Ending width
    pub width_end: ProtocolLinearDimension,
    // Section length
    pub length: ProtocolLinearDimension,
}

#[derive(Clone, Copy, PartialEq, Eq)]
/// Description of turning map section
pub struct ProtocolMapSectionDataTurn {
    // Starting width
    pub width_start: ProtocolLinearDimension,
    // Starting radius
    pub radius_start: ProtocolLinearDimension,
    // Ending width
    pub width_end: ProtocolLinearDimension,
    // Ending radius
    pub radius_end: ProtocolLinearDimension,
    // Section turning angle (always positive)
    pub angle: ProtocolAngle,
}

#[derive(Clone, Copy, PartialEq, Eq)]
/// Description of sloping map section
pub struct ProtocolMapSectionDataSlope {
    // Starting width
    pub width_start: ProtocolLinearDimension,
    // Ending width
    pub width_end: ProtocolLinearDimension,
    // Section length (flat)
    pub length: ProtocolLinearDimension,
    // Slope height (always positive)
    pub height: ProtocolLinearDimension,
}

#[derive(Clone, Copy, PartialEq, Eq)]
/// Data about a map section
pub enum ProtocolMapSectionData {
    /// Straight section
    Straight(ProtocolMapSectionDataStraight),
    /// Turn right section
    TurnRight(ProtocolMapSectionDataTurn),
    /// Turn left section
    TurnLeft(ProtocolMapSectionDataTurn),
    /// Climbing part of bridge section
    SlopeUp(ProtocolMapSectionDataSlope),
    /// Descending part of bridge section
    SlopeDown(ProtocolMapSectionDataSlope),
}

#[derive(Clone, Copy, PartialEq, Eq)]
/// Description of a map section
pub struct ProtocolMapSection {
    /// Section index
    pub index: usize,
    /// Section data
    pub data: ProtocolMapSectionData,
}

static MAP_START: &str = "MAP-START";
static MAP_SECTION: &str = "MAP-SECTION";
static MAP_END: &str = "MAP-END";
static RESET: &str = "RESET";
static START: &str = "START";
static PAUSE: &str = "PAUSE";
static RESTART: &str = "RESTART";
static DIRECT: &str = "DIRECT";

static STRAIGHT: &str = "STRAIGHT";
static LEFT: &str = "LEFT";
static RIGHT: &str = "RIGHT";
static UP: &str = "UP";
static DOWN: &str = "DOWN";

#[derive(Clone, Copy, PartialEq, Eq)]
/// Commands a bot can receive
pub enum BotCommand {
    /// Start of map data (with sections count)
    MapStart(usize),
    /// Individual map section
    MapSection(ProtocolMapSection),
    /// End of map data
    MapEnd,
    /// Reset (and initialize) robot hardware
    Reset,
    /// Start race (wait five seconds and start)
    Start,
    /// Pause bot (stop motors and wait for a [re]start)
    Pause,
    /// Restart race (wait 500ms and start)
    Restart,
    /// Directly apply motor power
    Direct(MotorsPowerData),
}

impl BotCommand {
    pub fn write(&self, buf: &mut ProtocolBuffer) {
        let mut index = 0;
        match self {
            BotCommand::MapStart(cmd) => {
                index = write_string(buf, index, MAP_START);
                index = append_separator(buf, index);
                index = write_i32(buf, index, *cmd as i32);
            }
            BotCommand::MapSection(cmd) => {
                index = write_string(buf, index, MAP_SECTION);
                index = append_separator(buf, index);
                index = write_i32(buf, index, cmd.index as i32);
                index = append_separator(buf, index);
                match cmd.data {
                    ProtocolMapSectionData::Straight(data) => {
                        index = write_string(buf, index, STRAIGHT);
                        index = append_separator(buf, index);
                        index = write_i32(buf, index, data.length);
                        index = append_separator(buf, index);
                        index = write_i32(buf, index, data.width_start);
                        index = append_separator(buf, index);
                        index = write_i32(buf, index, data.width_end);
                    }
                    ProtocolMapSectionData::TurnLeft(data) => {
                        index = write_string(buf, index, LEFT);
                        index = append_separator(buf, index);
                        index = write_i32(buf, index, data.angle);
                        index = append_separator(buf, index);
                        index = write_i32(buf, index, data.width_start);
                        index = append_separator(buf, index);
                        index = write_i32(buf, index, data.width_end);
                        index = append_separator(buf, index);
                        index = write_i32(buf, index, data.radius_start);
                        index = append_separator(buf, index);
                        index = write_i32(buf, index, data.radius_end);
                    }
                    ProtocolMapSectionData::TurnRight(data) => {
                        index = write_string(buf, index, RIGHT);
                        index = append_separator(buf, index);
                        index = write_i32(buf, index, data.angle);
                        index = append_separator(buf, index);
                        index = write_i32(buf, index, data.width_start);
                        index = append_separator(buf, index);
                        index = write_i32(buf, index, data.width_end);
                        index = append_separator(buf, index);
                        index = write_i32(buf, index, data.radius_start);
                        index = append_separator(buf, index);
                        index = write_i32(buf, index, data.radius_end);
                    }
                    ProtocolMapSectionData::SlopeUp(data) => {
                        index = write_string(buf, index, UP);
                        index = append_separator(buf, index);
                        index = write_i32(buf, index, data.length);
                        index = append_separator(buf, index);
                        index = write_i32(buf, index, data.height);
                        index = append_separator(buf, index);
                        index = write_i32(buf, index, data.width_start);
                        index = append_separator(buf, index);
                        index = write_i32(buf, index, data.width_end);
                    }
                    ProtocolMapSectionData::SlopeDown(data) => {
                        index = write_string(buf, index, DOWN);
                        index = append_separator(buf, index);
                        index = write_i32(buf, index, data.length);
                        index = append_separator(buf, index);
                        index = write_i32(buf, index, data.height);
                        index = append_separator(buf, index);
                        index = write_i32(buf, index, data.width_start);
                        index = append_separator(buf, index);
                        index = write_i32(buf, index, data.width_end);
                    }
                }
            }
            BotCommand::MapEnd => {
                index = write_string(buf, index, MAP_END);
            }
            BotCommand::Reset => {
                index = write_string(buf, index, RESET);
            }
            BotCommand::Start => {
                index = write_string(buf, index, START);
            }
            BotCommand::Pause => {
                index = write_string(buf, index, PAUSE);
            }
            BotCommand::Restart => {
                index = write_string(buf, index, RESTART);
            }
            BotCommand::Direct(cmd) => {
                index = write_string(buf, 0, DIRECT);
                index = append_separator(buf, index);
                index = write_i32(buf, index, cmd.back_left);
                index = append_separator(buf, index);
                index = write_i32(buf, index, cmd.back_right);
                index = append_separator(buf, index);
                index = write_i32(buf, index, cmd.front_left);
                index = append_separator(buf, index);
                index = write_i32(buf, index, cmd.front_right);
            }
        }
        append_end(buf, index);
    }

    pub fn parse(buf: &ProtocolBuffer) -> Result<Self, usize> {
        let mut index = 0;
        if let Ok(next) = match_string(buf, index, MAP_START) {
            index = next;
            index = match_separator(buf, index)?;
            let (size, next) = match_i32(buf, index)?;
            index = next;
            match_end(buf, index)?;
            Ok(BotCommand::MapStart(size as usize))
        } else if let Ok(next) = match_string(buf, index, MAP_SECTION) {
            index = next;
            index = match_separator(buf, index)?;
            let (section_index, next) = match_i32(buf, index)?;
            index = next;
            index = match_separator(buf, index)?;
            if let Ok(next) = match_string(buf, index, STRAIGHT) {
                index = next;
                index = match_separator(buf, index)?;
                let (length, next) = match_i32(buf, index)?;
                index = next;
                index = match_separator(buf, index)?;
                let (width_start, next) = match_i32(buf, index)?;
                index = next;
                index = match_separator(buf, index)?;
                let (width_end, next) = match_i32(buf, index)?;
                index = next;
                match_end(buf, index)?;
                Ok(BotCommand::MapSection(ProtocolMapSection {
                    index: section_index as usize,
                    data: ProtocolMapSectionData::Straight(ProtocolMapSectionDataStraight {
                        length,
                        width_start,
                        width_end,
                    }),
                }))
            } else if let Ok(next) = match_string(buf, index, LEFT) {
                index = next;
                index = match_separator(buf, index)?;
                let (angle, next) = match_i32(buf, index)?;
                index = next;
                index = match_separator(buf, index)?;
                let (width_start, next) = match_i32(buf, index)?;
                index = next;
                index = match_separator(buf, index)?;
                let (width_end, next) = match_i32(buf, index)?;
                index = next;
                index = match_separator(buf, index)?;
                let (radius_start, next) = match_i32(buf, index)?;
                index = next;
                index = match_separator(buf, index)?;
                let (radius_end, next) = match_i32(buf, index)?;
                index = next;
                match_end(buf, index)?;
                Ok(BotCommand::MapSection(ProtocolMapSection {
                    index: section_index as usize,
                    data: ProtocolMapSectionData::TurnLeft(ProtocolMapSectionDataTurn {
                        angle,
                        width_start,
                        width_end,
                        radius_start,
                        radius_end,
                    }),
                }))
            } else if let Ok(next) = match_string(buf, index, RIGHT) {
                index = next;
                index = match_separator(buf, index)?;
                let (angle, next) = match_i32(buf, index)?;
                index = next;
                index = match_separator(buf, index)?;
                let (width_start, next) = match_i32(buf, index)?;
                index = next;
                index = match_separator(buf, index)?;
                let (width_end, next) = match_i32(buf, index)?;
                index = next;
                index = match_separator(buf, index)?;
                let (radius_start, next) = match_i32(buf, index)?;
                index = next;
                index = match_separator(buf, index)?;
                let (radius_end, next) = match_i32(buf, index)?;
                index = next;
                match_end(buf, index)?;
                Ok(BotCommand::MapSection(ProtocolMapSection {
                    index: section_index as usize,
                    data: ProtocolMapSectionData::TurnRight(ProtocolMapSectionDataTurn {
                        angle,
                        width_start,
                        width_end,
                        radius_start,
                        radius_end,
                    }),
                }))
            } else if let Ok(next) = match_string(buf, index, UP) {
                index = next;
                index = match_separator(buf, index)?;
                let (length, next) = match_i32(buf, index)?;
                index = next;
                index = match_separator(buf, index)?;
                let (height, next) = match_i32(buf, index)?;
                index = next;
                index = match_separator(buf, index)?;
                let (width_start, next) = match_i32(buf, index)?;
                index = next;
                index = match_separator(buf, index)?;
                let (width_end, next) = match_i32(buf, index)?;
                index = next;
                match_end(buf, index)?;
                Ok(BotCommand::MapSection(ProtocolMapSection {
                    index: section_index as usize,
                    data: ProtocolMapSectionData::SlopeUp(ProtocolMapSectionDataSlope {
                        length,
                        height,
                        width_start,
                        width_end,
                    }),
                }))
            } else if let Ok(next) = match_string(buf, index, DOWN) {
                index = next;
                index = match_separator(buf, index)?;
                let (length, next) = match_i32(buf, index)?;
                index = next;
                index = match_separator(buf, index)?;
                let (height, next) = match_i32(buf, index)?;
                index = next;
                index = match_separator(buf, index)?;
                let (width_start, next) = match_i32(buf, index)?;
                index = next;
                index = match_separator(buf, index)?;
                let (width_end, next) = match_i32(buf, index)?;
                index = next;
                match_end(buf, index)?;
                Ok(BotCommand::MapSection(ProtocolMapSection {
                    index: section_index as usize,
                    data: ProtocolMapSectionData::SlopeDown(ProtocolMapSectionDataSlope {
                        length,
                        height,
                        width_start,
                        width_end,
                    }),
                }))
            } else {
                Err(index)
            }
        } else if let Ok(next) = match_string(buf, index, MAP_END) {
            index = next;
            match_end(buf, index)?;
            Ok(BotCommand::MapEnd)
        } else if let Ok(next) = match_string(buf, index, RESET) {
            index = next;
            match_end(buf, index)?;
            Ok(BotCommand::Reset)
        } else if let Ok(next) = match_string(buf, index, START) {
            index = next;
            match_end(buf, index)?;
            Ok(BotCommand::Start)
        } else if let Ok(next) = match_string(buf, index, PAUSE) {
            index = next;
            match_end(buf, index)?;
            Ok(BotCommand::Pause)
        } else if let Ok(next) = match_string(buf, index, RESTART) {
            index = next;
            match_end(buf, index)?;
            Ok(BotCommand::Restart)
        } else if let Ok(next) = match_string(buf, index, DIRECT) {
            index = next;
            index = match_separator(buf, index)?;
            let (back_left, next) = match_i32(buf, index)?;
            index = next;
            index = match_separator(buf, index)?;
            let (back_right, next) = match_i32(buf, index)?;
            index = next;
            index = match_separator(buf, index)?;
            let (front_left, next) = match_i32(buf, index)?;
            index = next;
            index = match_separator(buf, index)?;
            let (front_right, next) = match_i32(buf, index)?;
            index = next;
            match_end(buf, index)?;
            Ok(BotCommand::Direct(MotorsPowerData {
                back_left,
                back_right,
                front_left,
                front_right,
            }))
        } else {
            Err(index)
        }
    }
}

/// Time in milliseconds
pub type ProtocolTime = i32;

/// Completion of section (from 0 to 100)
pub type ProtocolCompletion = i32;

/// Side positioning in section (from -100 to 100)
pub type ProtocolSidePositioning = i32;

#[derive(Clone, Copy, PartialEq, Eq)]
/// Data for waiting state
pub struct ProtocolWaitingData {
    pub target: ProtocolTime,
    pub elapsed: ProtocolTime,
}

#[derive(Clone, Copy, PartialEq, Eq)]
/// Data for racing state
pub struct ProtocolRacingData {
    pub section: usize,
    pub completion_low: ProtocolCompletion,
    pub completion_high: ProtocolCompletion,
    pub positioning_left: ProtocolSidePositioning,
    pub positioning_right: ProtocolSidePositioning,
}

/// Data from all laser sensors
pub type ProtocolLaserData = [ProtocolLinearDimension; LASER_COUNT];

#[derive(Clone, Copy, PartialEq, Eq)]
/// Data from the IMU
pub struct ProtocolImuData {
    /// Bot Euler angle X
    pub rotation_x: ProtocolAngle,
    /// Bot Euler angle Y
    pub rotation_y: ProtocolAngle,
    /// Bot Euler angle Z
    pub rotation_z: ProtocolAngle,

    /// Bot linear acceleration X
    pub acceleration_x: ProtocolLinearAcceleration,
    /// Bot linear acceleration Y
    pub acceleration_y: ProtocolLinearAcceleration,
    /// Bot linear acceleration Z
    pub acceleration_z: ProtocolLinearAcceleration,

    /// Bot gravity acceleration X
    pub gravity_x: ProtocolLinearAcceleration,
    /// Bot gravity acceleration Y
    pub gravity_y: ProtocolLinearAcceleration,
    /// Bot gravity acceleration Z
    pub gravity_z: ProtocolLinearAcceleration,
}

#[derive(Clone, Copy, PartialEq, Eq)]
/// Bot status
pub enum ProtocolBotStatus {
    /// No valid map yet
    InvalidMap,
    /// Initialization or reset failed
    DeviceError,
    /// Stopped, waiting more commands
    Stopped,
    /// Waiting to [re]start race
    Waiting(ProtocolWaitingData),
    /// Racing
    Racing(ProtocolRacingData),
}

#[derive(Clone, Copy)]
pub struct ProtocolLogLineData {
    pub length: usize,
    pub message: [u8; MAX_LOG_LINE_SIZE],
}

impl PartialEq for ProtocolLogLineData {
    fn eq(&self, other: &Self) -> bool {
        if self.length != other.length {
            return false;
        }
        for i in 0..self.length {
            if self.message[i] != other.message[i] {
                return false;
            }
        }
        true
    }
}

impl Eq for ProtocolLogLineData {}

#[derive(Clone, Copy, PartialEq, Eq)]
/// Event messages the bot can emit
pub enum BotEvent {
    Status(ProtocolBotStatus),
    Lasers(ProtocolLaserData),
    Imu(ProtocolImuData),
    Log(ProtocolLogLineData),
}

static STATUS: &str = "STATUS";
static LASERS: &str = "LASERS";
static IMU: &str = "IMU";
static LOG: &str = "LOG";

static INVALID_MAP: &str = "INVALID-MAP";
static DEVICE_ERROR: &str = "DEVICE-ERROR";
static STOPPED: &str = "STOPPED";
static WAITING: &str = "WAITING";
static RACING: &str = "RACING";

impl BotEvent {
    pub fn write(&self, buf: &mut ProtocolBuffer) {
        let mut index = 0;
        match self {
            BotEvent::Status(evt) => {
                index = write_string(buf, index, STATUS);
                index = append_separator(buf, index);
                match evt {
                    ProtocolBotStatus::InvalidMap => {
                        index = write_string(buf, index, INVALID_MAP);
                    }
                    ProtocolBotStatus::DeviceError => {
                        index = write_string(buf, index, DEVICE_ERROR);
                    }
                    ProtocolBotStatus::Stopped => {
                        index = write_string(buf, index, STOPPED);
                    }
                    ProtocolBotStatus::Waiting(data) => {
                        index = write_string(buf, index, WAITING);
                        index = append_separator(buf, index);
                        index = write_i32(buf, index, data.target);
                        index = append_separator(buf, index);
                        index = write_i32(buf, index, data.elapsed);
                    }
                    ProtocolBotStatus::Racing(data) => {
                        index = write_string(buf, index, RACING);
                        index = append_separator(buf, index);
                        index = write_i32(buf, index, data.section as i32);
                        index = append_separator(buf, index);
                        index = write_i32(buf, index, data.completion_low);
                        index = append_separator(buf, index);
                        index = write_i32(buf, index, data.completion_high);
                        index = append_separator(buf, index);
                        index = write_i32(buf, index, data.positioning_left);
                        index = append_separator(buf, index);
                        index = write_i32(buf, index, data.positioning_right);
                    }
                }
            }
            BotEvent::Lasers(evt) => {
                index = write_string(buf, index, LASERS);
                for laser in evt {
                    index = append_separator(buf, index);
                    index = write_i32(buf, index, *laser);
                }
            }
            BotEvent::Imu(evt) => {
                index = write_string(buf, index, IMU);
                index = append_separator(buf, index);
                index = write_i32(buf, index, evt.rotation_x);
                index = append_separator(buf, index);
                index = write_i32(buf, index, evt.rotation_y);
                index = append_separator(buf, index);
                index = write_i32(buf, index, evt.rotation_z);
                index = append_separator(buf, index);
                index = write_i32(buf, index, evt.acceleration_x);
                index = append_separator(buf, index);
                index = write_i32(buf, index, evt.acceleration_y);
                index = append_separator(buf, index);
                index = write_i32(buf, index, evt.acceleration_z);
                index = append_separator(buf, index);
                index = write_i32(buf, index, evt.gravity_x);
                index = append_separator(buf, index);
                index = write_i32(buf, index, evt.gravity_y);
                index = append_separator(buf, index);
                index = write_i32(buf, index, evt.gravity_z);
            }
            BotEvent::Log(evt) => {
                index = write_string(buf, index, LOG);
                index = append_separator(buf, index);
                for i in 0..evt.length {
                    index = append_code(buf, index, evt.message[i]);
                }
            }
        }
        append_end(buf, index);
    }

    pub fn parse(buf: &ProtocolBuffer) -> Result<Self, usize> {
        let mut index = 0;
        if let Ok(next) = match_string(buf, index, STATUS) {
            index = next;
            index = match_separator(buf, index)?;
            if let Ok(next) = match_string(buf, index, INVALID_MAP) {
                index = next;
                match_end(buf, index)?;
                Ok(BotEvent::Status(ProtocolBotStatus::InvalidMap))
            } else if let Ok(next) = match_string(buf, index, DEVICE_ERROR) {
                index = next;
                match_end(buf, index)?;
                Ok(BotEvent::Status(ProtocolBotStatus::DeviceError))
            } else if let Ok(next) = match_string(buf, index, STOPPED) {
                index = next;
                match_end(buf, index)?;
                Ok(BotEvent::Status(ProtocolBotStatus::Stopped))
            } else if let Ok(next) = match_string(buf, index, WAITING) {
                index = next;
                index = match_separator(buf, index)?;
                let (target, next) = match_i32(buf, index)?;
                index = next;
                index = match_separator(buf, index)?;
                let (elapsed, next) = match_i32(buf, index)?;
                index = next;
                match_end(buf, index)?;
                Ok(BotEvent::Status(ProtocolBotStatus::Waiting(
                    ProtocolWaitingData { target, elapsed },
                )))
            } else if let Ok(next) = match_string(buf, index, RACING) {
                index = next;
                index = match_separator(buf, index)?;
                let (section, next) = match_i32(buf, index)?;
                index = next;
                index = match_separator(buf, index)?;
                let (completion_low, next) = match_i32(buf, index)?;
                index = next;
                index = match_separator(buf, index)?;
                let (completion_high, next) = match_i32(buf, index)?;
                index = next;
                index = match_separator(buf, index)?;
                let (positioning_left, next) = match_i32(buf, index)?;
                index = next;
                index = match_separator(buf, index)?;
                let (positioning_right, next) = match_i32(buf, index)?;
                index = next;
                match_end(buf, index)?;
                Ok(BotEvent::Status(ProtocolBotStatus::Racing(
                    ProtocolRacingData {
                        section: section as usize,
                        completion_low,
                        completion_high,
                        positioning_left,
                        positioning_right,
                    },
                )))
            } else {
                Err(index)
            }
        } else if let Ok(next) = match_string(buf, index, LASERS) {
            index = next;
            let mut data: ProtocolLaserData = [0; LASER_COUNT];
            for i in 0..LASER_COUNT {
                index = match_separator(buf, index)?;
                let (laser, next) = match_i32(buf, index)?;
                index = next;
                data[i] = laser;
            }
            Ok(BotEvent::Lasers(data))
        } else if let Ok(next) = match_string(buf, index, IMU) {
            index = next;
            index = match_separator(buf, index)?;
            let (rotation_x, next) = match_i32(buf, index)?;
            index = next;
            index = match_separator(buf, index)?;
            let (rotation_y, next) = match_i32(buf, index)?;
            index = next;
            index = match_separator(buf, index)?;
            let (rotation_z, next) = match_i32(buf, index)?;
            index = next;
            index = match_separator(buf, index)?;
            let (acceleration_x, next) = match_i32(buf, index)?;
            index = next;
            index = match_separator(buf, index)?;
            let (acceleration_y, next) = match_i32(buf, index)?;
            index = next;
            index = match_separator(buf, index)?;
            let (acceleration_z, next) = match_i32(buf, index)?;
            index = next;
            index = match_separator(buf, index)?;
            let (gravity_x, next) = match_i32(buf, index)?;
            index = next;
            index = match_separator(buf, index)?;
            let (gravity_y, next) = match_i32(buf, index)?;
            index = next;
            index = match_separator(buf, index)?;
            let (gravity_z, next) = match_i32(buf, index)?;
            index = next;
            match_end(buf, index)?;
            Ok(BotEvent::Imu(ProtocolImuData {
                rotation_x,
                rotation_y,
                rotation_z,
                acceleration_x,
                acceleration_y,
                acceleration_z,
                gravity_x,
                gravity_y,
                gravity_z,
            }))
        } else if let Ok(next) = match_string(buf, index, LOG) {
            index = next;
            index = match_separator(buf, index)?;
            let mut data = ProtocolLogLineData {
                length: 0,
                message: [0; MAX_LOG_LINE_SIZE],
            };
            for i in 0..MAX_LOG_LINE_SIZE {
                let code = buf[index];
                if code != CODE_END {
                    data.message[i] = code;
                    index += 1;
                } else {
                    data.length = i;
                    break;
                }
            }
            match_end(buf, index)?;
            Ok(BotEvent::Log(data))
        } else {
            Err(index)
        }
    }
}

pub trait CommandReceiver {
    fn poll() -> Option<BotCommand>;
}
pub trait CommandEmitter {
    fn emit(cmd: BotCommand);
}

pub trait EventReceiver {
    fn poll() -> Option<BotEvent>;
}
pub trait EventEmitter {
    fn emit(cmd: BotEvent);
}
