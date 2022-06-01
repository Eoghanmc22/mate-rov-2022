use common::CommunicationError;
use crate::buffer::Buffer;

pub fn write_baud(buffer: &mut [u8]) -> Result<&mut [u8], CommunicationError> {
    let mut buffer = Buffer::new(buffer);
    buffer.push(0xAA)?;
    Ok(buffer.into_buffer())
}