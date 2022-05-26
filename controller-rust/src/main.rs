#![no_std]
#![no_main]

use arduino_hal::prelude::*;
use arrayvec::{ArrayString, ArrayVec};
use embedded_hal::prelude::*;
use panic_halt as _;
use common::find_frame;

//todo look into use of framed crate

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    let mut usb = arduino_hal::default_serial!(dp, pins, common::BAUD_RATE_PC);
    let mut nano_serial = arduino_hal::Usart::new(dp.USART1, pins.d19, pins.d18.into_output(), common::BAUD_RATE_NANO.into_baudrate());

    let mut tx_buffer = ArrayString::<200>::new();
    let mut rx_buffer = ArrayVec::<u8, 200>::new();

    loop {
        while let Ok(byte) = nano_serial.read() {
            usb.write_byte(byte);

            if byte == common::MSG_END as u8 {
                if !tx_buffer.is_empty() {
                    usb.write_str(&tx_buffer).unwrap();
                    tx_buffer.clear();
                }
                usb.write_byte(common::MSG_START as u8);
            }
        }

        while let Ok(byte) = usb.read() {
            rx_buffer.push(byte);
        }

        let new_length = parse_commands(&mut rx_buffer);
        unsafe { rx_buffer.set_len(new_length) }
    }
}

fn parse_commands(buffer: &mut [u8]) -> usize {
    let mut offset = 0;

    while let Some(frame) = find_frame(&buffer[offset..]) {
        process_command(frame);

        offset += frame.len() + 1;
    }

    let range = offset..buffer.len();
    buffer.copy_within(range.clone(), 0);
    range.len()
}

fn process_command(command: &str) {
    write_message(ufmt::)
}

fn write_message<const CAP: usize>(message: &str, buffer: &mut ArrayString<CAP>) {
    buffer.push(common::MSG_START);
    buffer.push_str(message);
    buffer.push(common::MSG_END);
}


