#![no_std]

use postcard::flavors::{Cobs, Slice};
use serde::{Serialize, Deserialize};
use crate::crc::Crc;

pub mod controller;
pub mod crc;

// other vals can have less error?
pub const BAUD_RATE_PC : u32 = 1000000;//921600;//460800;//115200;
pub const BAUD_RATE_NANO : u32 = 57600;

#[derive(Debug)]
pub enum CommunicationError {
    BadData,
    BadCheckSum(u16, u16),
    EOF,
    BufferFull,
    TooSmall,
    InternalError
}

impl From<postcard::Error> for CommunicationError {
    fn from(error: postcard::Error) -> Self {
        use postcard::Error::*;

        match error {
            DeserializeBadVarint | DeserializeBadBool | DeserializeBadChar | DeserializeBadUtf8 | DeserializeBadOption | DeserializeBadEnum | DeserializeBadEncoding => {
                CommunicationError::BadData
            }
            DeserializeUnexpectedEnd => {
                CommunicationError::EOF
            }
            SerializeBufferFull => {
                CommunicationError::BufferFull
            }
            _ => {
                CommunicationError::InternalError
            }
        }
    }
}

#[cfg(test)]
mod test {
    use core::mem::MaybeUninit;
    use crate::controller::{DownstreamMessage, VelocityData};
    use crate::{read, write};

    #[test]
    fn test() {
        let mut buffer : [u8; 200] = unsafe { MaybeUninit::uninit().assume_init() };

        let command = DownstreamMessage::VelocityDataMessage(VelocityData {
            direction: (1.0, 2.0, 3.0),
            yaw_split: 0.5
        });

        let buffer2 = write(&command, &mut buffer).unwrap();
        let received = read::<DownstreamMessage>(buffer2).unwrap(); //

        match received {
            DownstreamMessage::VelocityDataMessage(data) => {
                assert_eq!(data.direction.0, 1.0);
                assert_eq!(data.direction.1, 2.0);
                assert_eq!(data.direction.2, 3.0);
                assert_eq!(data.yaw_split, 0.5);
            }
            _ => { panic!() }
        }
    }
}

pub fn write<'a, S: Serialize>(obj: &S, out: &'a mut [u8]) -> Result<&'a mut [u8], CommunicationError> {
    postcard::serialize_with_flavor(obj, Crc::new(Cobs::try_new(Slice::new(out)).map_err(CommunicationError::from)?)).map_err(CommunicationError::from)
}

pub fn read<'a, D: Deserialize<'a>>(buffer: &'a mut [u8]) -> Result<D, CommunicationError> {
    let read = postcard_cobs::decode_in_place(buffer).map_err(|_| CommunicationError::BadData)?;
    if read > 3 {
        let data = &buffer[..read - 3];
        let crc = u16::from_le_bytes((&buffer[read - 3..read - 1]).try_into().unwrap());

        let checksum = crate::crc::CRC.checksum(data);
        if checksum == crc {
            postcard::from_bytes(data).map_err(CommunicationError::from)
        } else {
            Err(CommunicationError::BadCheckSum(checksum, crc))
        }
    } else {
        Err(CommunicationError::EOF)
    }
}

pub fn end_of_frame(byte: u8) -> bool {
    byte == 0x00
}