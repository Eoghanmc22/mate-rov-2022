use common::CommunicationError;

pub struct Buffer<'a> {
    buffer: &'a mut [u8],
    index: usize,
}

impl<'a> Buffer<'a> {
    pub fn new(buffer: &'a mut [u8]) -> Self {
        Buffer {
            buffer,
            index: 0
        }
    }

    pub fn push(&mut self, byte: u8) -> Result<(), CommunicationError> {
        if self.index >= self.buffer.len() {
            return Err(CommunicationError::BufferFull);
        }

        self.buffer[self.index] = byte;
        self.index += 1;

        Ok(())
    }

    pub fn into_buffer(self) -> &'a mut [u8] {
        &mut self.buffer[..self.index]
    }
}
