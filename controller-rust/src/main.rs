#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

mod state;
mod atomic;
mod sealed;
mod spsc;

use core::cell::RefCell;
use core::mem;
use arduino_hal::prelude::*;
use embedded_hal::prelude::*;
use embedded_hal::serial::Write;
use heapless::{String, Vec, Deque};
use nb::block;
use common::controller::{DownstreamMessage, UpstreamMessage};
use crate::state::State;

use core::panic::PanicInfo;
use core::sync::atomic::Ordering;
use arduino_hal::{default_serial, Peripherals, pins, Usart};
use arduino_hal::hal::port::{PD2, PD3, PE0, PE1};
use arduino_hal::hal::usart::Event;
use arduino_hal::port::mode::{Input, Output};
use arduino_hal::port::Pin;
use arduino_hal::usart::UsartReader;
use avr_device::atmega2560::{USART0, USART1};
use avr_device::interrupt;
use avr_device::interrupt::Mutex;
use crate::spsc::{Consumer, Producer, Queue};
use ufmt::uwriteln;

#[inline(never)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    interrupt::disable();

    let dp = unsafe { Peripherals::steal() };
    let pins = pins!(dp);
    let mut usb = default_serial!(dp, pins, common::BAUD_RATE_CTRL);

    let location = info.location().unwrap();

    loop {
        uwriteln!(usb, "Panicked at {}:{} in {}", location.line(), location.column(), location.file());
        atomic::compiler_fence(Ordering::SeqCst);
    }
}

static USB_SERIAL_RX: Mutex<RefCell<Option<UsartReader<USART0, Pin<Input, PE0>, Pin<Output, PE1>>>>> = Mutex::new(RefCell::new(None));
static NANO_SERIAL_RX: Mutex<RefCell<Option<UsartReader<USART1, Pin<Input, PD2>, Pin<Output, PD3>>>>> = Mutex::new(RefCell::new(None));

static mut NANO_QUEUE: Queue<u8, 64> = Queue::new();
static NANO_PRODUCER: Mutex<RefCell<Option<Producer<u8, 64>>>> = Mutex::new(RefCell::new(None));
static NANO_CONSUMER: Mutex<RefCell<Option<Consumer<u8, 64>>>> = Mutex::new(RefCell::new(None));

static mut USB_QUEUE: Queue<u8, 256> = Queue::new();
static USB_PRODUCER: Mutex<RefCell<Option<Producer<u8, 256>>>> = Mutex::new(RefCell::new(None));
static USB_CONSUMER: Mutex<RefCell<Option<Consumer<u8, 256>>>> = Mutex::new(RefCell::new(None));

#[arduino_hal::entry]
fn main() -> ! {
    let mut usb_buffer = Vec::<u8, { mem::size_of::<DownstreamMessage>() + 5 }>::new();

    let dp = Peripherals::take().unwrap();
    let pins = pins!(dp);
    let mut usb = default_serial!(dp, pins, common::BAUD_RATE_CTRL);
    let mut nano = Usart::new(dp.USART1, pins.d19, pins.d18.into_output(), common::BAUD_RATE_NANO.into_baudrate());

    usb.listen(Event::RxComplete);
    //nano.listen(Event::RxComplete);

    let (nano_reader, _) = nano.split();
    let (usb_reader, mut usb_writer) = usb.split();

    write_message(&UpstreamMessage::Init, &mut usb_writer);

    interrupt::free(|cs| {
        USB_SERIAL_RX.borrow(cs).borrow_mut().replace(usb_reader);
        NANO_SERIAL_RX.borrow(cs).borrow_mut().replace(nano_reader);

        let (nano_producer, nano_consumer) = unsafe { NANO_QUEUE.split() };
        NANO_PRODUCER.borrow(cs).replace(Some(nano_producer));
        NANO_CONSUMER.borrow(cs).replace(Some(nano_consumer));

        let (usb_producer, usb_consumer) = unsafe { USB_QUEUE.split() };
        USB_PRODUCER.borrow(cs).replace(Some(usb_producer));
        USB_CONSUMER.borrow(cs).replace(Some(usb_consumer));
    });

    // Enable interrupts globally
    unsafe { interrupt::enable() };

    let mut state = State::default();

    loop {
        // process data from computer
        let byte = interrupt::free(|cs| {
            let mut usb_consumer = USB_CONSUMER.borrow(cs).borrow_mut();
            usb_consumer.as_mut().unwrap().dequeue()
        });

        if let Some(byte) = byte {
            if let Ok(()) = usb_buffer.push(byte) {
                if common::end_of_frame(&byte) {
                    match common::read(&mut usb_buffer) {
                        Ok(message) =>  {
                            state.update(message);
                            write_message(&UpstreamMessage::Ack, &mut usb_writer);
                        }
                        Err(e) => {
                            // data was not received correctly
                            write_message(&UpstreamMessage::BadP(e), &mut usb_writer);
                        }
                    }
                    usb_buffer.clear();
                }
            } else {
                // data was not received correctly
                write_message(&UpstreamMessage::BadO, &mut usb_writer);
                usb_buffer.clear();
            }
        }

        /*// forward data from nano
        // maybe optimize this by using a buffer?
        let byte = interrupt::free(|cs| {
            let mut nano_consumer = NANO_CONSUMER.borrow(cs).borrow_mut();
            nano_consumer.as_mut().unwrap().dequeue()
        });

        if let Some(byte) = byte {
            write_message(&UpstreamMessage::IMUStream(byte), &mut usb_writer);
        }*/
    }
}

//TODO CLEANUP

#[avr_device::interrupt(atmega2560)]
#[allow(non_snake_case)]
fn USART0_RX() {
    interrupt::free(|cs| {
        let mut usb = USB_SERIAL_RX.borrow(cs).borrow_mut();
        let mut usb_producer = USB_PRODUCER.borrow(cs).borrow_mut();

        while let Ok(byte) = usb.as_mut().unwrap().read() {
            let _ = usb_producer.as_mut().unwrap().enqueue(byte).unwrap();
        }
    });
}

#[avr_device::interrupt(atmega2560)]
#[allow(non_snake_case)]
fn USART1_RX() {
    interrupt::free(|cs| {
        let mut nano_rx = NANO_SERIAL_RX.borrow(cs).borrow_mut();
        let mut nano_producer = NANO_PRODUCER.borrow(cs).borrow_mut();

        while let Ok(byte) = nano_rx.as_mut().unwrap().read() {
            let _ = nano_producer.as_mut().unwrap().enqueue(byte).unwrap();
        }
    });
}

/// This function is unsafe when called from an interrupt handler
fn write_message(message: &UpstreamMessage, serial: &mut impl Write<u8>) {
    static mut OUT_BUFFER: [u8; 200] = [0; 200];

    let buffer = unsafe { &mut OUT_BUFFER };
    if let Ok(buffer) = common::write(message, buffer) {
        for &mut byte in buffer {
            let result = block!(serial.write(byte));

            if result.is_err() {
                break;
            }
        }
    }
}
