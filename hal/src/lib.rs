/// Number of laser sensors
pub const LASER_COUNT: usize = 20;

/// LinearDimension in mm
pub type LinearDimension = f32;

/// Time in ms
pub type Time = f32;

/// Acceleration in mm/s2
pub type Acceleration = f32;

/// Angle in radians, positive clockwise
pub type Angle = f32;

/// Adimensional value (from -1 to 1 or from 0 to 1)
pub type Dim = f32;

/// Data from all laser sensors
pub type LaserData = [LinearDimension; LASER_COUNT];

/// Motor power, range from -1 to +1
pub type MotorPower = Dim;

#[derive(Clone, Copy)]
/// Data from the IMU
pub struct ImuData {
    /// Bot heading
    pub heading: Angle,
    /// Bot pitch
    pub pitch: Angle,
    /// Bot roll
    pub roll: Angle,
    /// Bot linear acceleration along x axis (mm/s2, decoupled from gravity)
    pub acceleration_x: Acceleration,
    /// Bot linear acceleration along y axis (mm/s2, decoupled from gravity)
    pub acceleration_y: Acceleration,
    /// Bot linear acceleration along z axis (mm/s2, decoupled from gravity)
    pub acceleration_z: Acceleration,
}

pub const PROTOCOL_BUFFER_SIZE: usize = 256;
pub type ProtocolBuffer = [u8; PROTOCOL_BUFFER_SIZE];

pub fn new_protocol_buffer() -> ProtocolBuffer {
    [0 as u8; PROTOCOL_BUFFER_SIZE]
}

/// Abstraction over physical devices
pub trait DeviceHal {
    /// Initialize bot hardware
    fn init() -> Result<(), ()>;

    /// Read IMU data
    fn read_imu() -> Result<ImuData, ()>;

    /// Read laser data
    fn read_lasers() -> Result<LaserData, ()>;

    /// Set motor power
    fn set_motor_power(
        back_left: MotorPower,
        back_right: MotorPower,
        front_left: MotorPower,
        front_right: MotorPower,
    );

    /// Poll serial line for data
    fn poll() -> Option<ProtocolBuffer>;

    /// Send data on the serial line
    fn send(data: ProtocolBuffer);
}
