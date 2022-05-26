#![no_std]

pub const BAUD_RATE_PC : u32 = 115200;
pub const BAUD_RATE_NANO : u32 = 57600;

pub const MSG_START : char = '`';
pub const MSG_END: char = '\n';

pub const VELOCITY_COMMAND : char = 'V';

pub fn find_frame(buffer: &[u8]) -> Option<&str> {
    let mut iter = buffer.iter();
    let start = iter.position(|&byte| byte == MSG_START as u8);
    let end = iter.position(|&byte| byte == MSG_END as u8);

    start.zip(end)
        .filter(|(start, end)| end > start)
        .map(|(start, end)| core::str::from_utf8(&buffer[start..end]).unwrap())
}
