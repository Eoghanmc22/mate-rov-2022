#![no_std]
#![no_main]

mod state;

use arduino_hal::prelude::*;
use embedded_hal::prelude::*;
use embedded_hal::serial::Write;
use heapless::{String, Vec};
use nb::block;
use common::controller::UpstreamMessage;
use crate::state::State;

use core::panic::PanicInfo;
use core::sync::atomic;
use core::sync::atomic::Ordering;
use arduino_hal::hal::wdt;
use arduino_hal::{default_serial, Peripherals, pins, Usart};

#[inline(never)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    use core::fmt::Write;

    avr_device::interrupt::disable();

    let dp = unsafe { Peripherals::steal() };
    let pins = pins!(dp);
    let mut usb = default_serial!(dp, pins, common::BAUD_RATE_PC);

    let mut buffer = String::<200>::new();
    let mut out_buffer = [0; 210];

    if let Ok(()) = write!(buffer, "{:?}", info) {
        write_message(&UpstreamMessage::Panic(&buffer), &mut usb, &mut out_buffer);
    } else {
        let _ = usb.write_str("panic");
    }
    usb.flush();

    loop {
        atomic::compiler_fence(Ordering::SeqCst);
    }
}

#[arduino_hal::entry]
fn main() -> ! {
    let mut rx_buffer = Vec::<u8, 500>::new();
    let mut out_buffer = [0; 200];

    let dp = Peripherals::take().unwrap();
    let pins = pins!(dp);
    let mut usb = default_serial!(dp, pins, common::BAUD_RATE_PC);
    let mut nano_serial = Usart::new(dp.USART1, pins.d19, pins.d18.into_output(), common::BAUD_RATE_NANO.into_baudrate());

    write_message(&UpstreamMessage::Init, &mut usb, &mut out_buffer);

    let mut state = State::default();

    let mut watchdog = wdt::Wdt::new(dp.WDT, &dp.CPU.mcusr);
    watchdog.start(wdt::Timeout::Ms16).unwrap();

    loop {
        if let Ok(byte) = nano_serial.read() {
            write_message(&UpstreamMessage::IMUStream(byte), &mut usb, &mut out_buffer);
        }

        while let Ok(byte) = usb.read() {
            if let Ok(()) = rx_buffer.push(byte) {
                if common::end_of_frame(byte) {
                    if let Ok(message) = common::read(&mut rx_buffer) {
                        state.update(message);
                        write_message(&UpstreamMessage::Ack, &mut usb, &mut out_buffer);
                    } else {
                        write_message(&UpstreamMessage::Bad, &mut usb, &mut out_buffer);
                    }
                    rx_buffer.clear();
                }
            } else {
                rx_buffer.clear();
            }
        }

        watchdog.feed();
    }
}

fn write_message(message: &UpstreamMessage, serial: &mut impl Write<u8>, buffer: &mut [u8]) {
    if let Ok(buffer) = common::write(message, buffer) {
        for &mut byte in buffer {
            let result = block!(serial.write(byte));

            if result.is_err() {
                break;
            }
        }
    }
}