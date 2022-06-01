#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

mod state;
mod atomic;
mod sealed;
mod spsc;

use core::cell::RefCell;
use core::mem;
use core::ops::DerefMut;
use arduino_hal::prelude::*;
use embedded_hal::prelude::*;
use embedded_hal::serial::Write;
use heapless::Vec;
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

    // This is safe because no other code can be running at the same time
    let dp = unsafe { Peripherals::steal() };
    let pins = pins!(dp);
    let mut usb = default_serial!(dp, pins, common::BAUD_RATE_CTRL);

    let location = info.location().unwrap();
    loop {
        let _ = uwriteln!(usb, "Panicked at {}:{} in {}", location.line(), location.column(), location.file());
        atomic::compiler_fence(Ordering::SeqCst);
    }
}

static USB_READER_SERIAL: Mutex<RefCell<Option<UsartReader<USART0, Pin<Input, PE0>, Pin<Output, PE1>>>>> = Mutex::new(RefCell::new(None));

static mut USB_READ_QUEUE: Queue<u8, 256> = Queue::new();
static USB_READ_PRODUCER: Mutex<RefCell<Option<Producer<u8, 256>>>> = Mutex::new(RefCell::new(None));
static USB_READ_CONSUMER: Mutex<RefCell<Option<Consumer<u8, 256>>>> = Mutex::new(RefCell::new(None));

#[arduino_hal::entry]
fn main() -> ! {
    // This buffer will hold partially received packets
    let mut usb_buffer = Vec::<u8, { mem::size_of::<DownstreamMessage>() + 5 }>::new();

    // Setup up serial communication
    let dp = Peripherals::take().unwrap();
    let pins = pins!(dp);
    let mut usb = default_serial!(dp, pins, common::BAUD_RATE_CTRL);

    // To improve reliability, we need to handle serial data as soon as it is received
    usb.listen(Event::RxComplete);

    let (usb_reader, mut usb_writer) = usb.split();

    // Initialize globals
    interrupt::free(|cs| {
        USB_READER_SERIAL.borrow(cs).borrow_mut().replace(usb_reader);

        let (usb_read_producer, usb_read_consumer) = unsafe { USB_READ_QUEUE.split() };
        USB_READ_PRODUCER.borrow(cs).replace(Some(usb_read_producer));
        USB_READ_CONSUMER.borrow(cs).replace(Some(usb_read_consumer));
    });

    // Enable interrupts globally
    unsafe { interrupt::enable() };

    // Notify the connected pc that we are ready to receive data
    write_message(&UpstreamMessage::Init, &mut usb_writer);

    let mut state = State::default();
    loop {
        // process data from computer
        loop {
            // Get the next byte from the queue
            let byte = interrupt::free(|cs| {
                if let Some(ref mut usb_consumer) = USB_READ_CONSUMER.borrow(cs).borrow_mut().deref_mut() {
                    usb_consumer.dequeue()
                } else {
                    None
                }
            });

            // Process that byte
            if let Some(byte) = byte {
                // Add that byte to the buffer
                if let Ok(()) = usb_buffer.push(byte) {
                    // If that byte signals the end of a packet we needed to parse the packet
                    if common::end_of_frame(&byte) {
                        match common::read(&mut usb_buffer) {
                            Ok(message) => {
                                // Update the robot's state and send acknowledgement
                                state.update(message);
                                write_message(&UpstreamMessage::Ack, &mut usb_writer);
                            }
                            Err(e) => {
                                // data was corrupted during transmission
                                write_message(&UpstreamMessage::BadP(e), &mut usb_writer);
                            }
                        }

                        // Clear the packet buffer so we can receive the next packet
                        usb_buffer.clear();
                    }
                } else {
                    // data buffer was over run, a seperator byte was mis-received
                    write_message(&UpstreamMessage::BadO, &mut usb_writer);
                    usb_buffer.clear();
                }
            } else {
                // No more new data
                break;
            }
        }
    }
}

#[avr_device::interrupt(atmega2560)]
#[allow(non_snake_case)]
fn USART0_RX() {
    interrupt::free(|cs| {
        // Access globals
        if let Some(ref mut usb) = USB_READER_SERIAL.borrow(cs).borrow_mut().deref_mut() {
            if let Some(ref mut usb_producer) = USB_READ_PRODUCER.borrow(cs).borrow_mut().deref_mut() {
                // Read all available data
                while let Ok(byte) = usb.read() {
                    //todo remove unwrap
                    let _ = usb_producer.enqueue(byte).unwrap();
                }
            }
        }
    });
}

/// This function is unsafe when called from an interrupt handler
fn write_message(message: &UpstreamMessage, serial: &mut impl Write<u8>) {
    static mut OUT_BUFFER: [u8; 200] = [0; 200];

    // Retrieve a temporary buffer and encode the packet into it
    let buffer = unsafe { &mut OUT_BUFFER };
    if let Ok(buffer) = common::write(message, buffer) {
        // write each byte to serial
        for &mut byte in buffer {
            let _ = block!(serial.write(byte));
        }
    }
}
