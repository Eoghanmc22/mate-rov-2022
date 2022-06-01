use common::CommunicationError;

// Address, Command
pub struct Motor(u8, u8);

pub const SABERTOOTH_A: u8 = 128;
pub const SABERTOOTH_B: u8 = 129;

pub const MOTOR_RIGHT: Motor = Motor(SABERTOOTH_A, 0);
pub const MOTOR_LEFT: Motor = Motor(SABERTOOTH_A, 4);
pub const MOTOR_VERTICAL: Motor = Motor(SABERTOOTH_B, 0);
pub const MOTOR_STRAFING: Motor = Motor(SABERTOOTH_B, 4);

pub fn write_init(buffer: &mut [u8]) -> Result<&mut [u8], CommunicationError> {
    let mut buffer = Buffer::new(buffer);
    buffer.write_byte(0xAA)?;
    Ok(buffer.into_buffer())
}

pub fn write_speed(buffer: &mut [u8], motor: Motor, speed: i8) -> Result<&mut [u8], CommunicationError> {
    let mut buffer = Buffer::new(buffer);
    buffer.write_byte(motor.0)?;

    if speed > 0 {
        buffer.write_byte(motor.1)?;
        buffer.write_byte(speed as u8)?;
    } else {
        buffer.write_byte(motor.1 + 1)?;
        buffer.write_byte(-(speed.max(-127)) as u8)?;
    }

    buffer.write_checksum()
}

pub fn write_min_voltage(buffer: &mut [u8], address: u8, voltage: f32) -> Result<&mut [u8], CommunicationError> {
    assert!(voltage >= 6.0);

    let mut buffer = Buffer::new(buffer);
    buffer.write_byte(address)?;
    buffer.write_byte(2)?;
    buffer.write_byte(((voltage - 6.0) * 5.0) as u8)?;

    buffer.write_checksum()
}

pub fn write_max_voltage(buffer: &mut [u8], address: u8, voltage: f32) -> Result<&mut [u8], CommunicationError> {
    let mut buffer = Buffer::new(buffer);
    buffer.write_byte(address)?;
    buffer.write_byte(3)?;
    buffer.write_byte((voltage * 5.12) as u8)?;

    buffer.write_checksum()
}


pub struct Buffer<'a> {
    buffer: &'a mut [u8],
    index: usize,
    sum: u8
}

impl<'a> Buffer<'a> {
    pub fn new(buffer: &'a mut [u8]) -> Self {
        Buffer {
            buffer,
            index: 0,
            sum: 0
        }
    }

    pub fn write_byte(&mut self, byte: u8) -> Result<(), CommunicationError> {
        if self.index >= self.buffer.len() {
            return Err(CommunicationError::BufferFull);
        }

        self.buffer[self.index] = byte;
        self.index += 1;

        self.sum = self.sum.wrapping_add(byte);

        Ok(())
    }

    pub fn write_checksum(mut self) -> Result<&'a mut [u8], CommunicationError> {
        self.write_byte(self.sum & 0b01111111)?;
        Ok(self.into_buffer())
    }

    pub fn into_buffer(self) -> &'a mut [u8] {
        &mut self.buffer[..self.index]
    }
}
