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
use heapless::{String, Vec, Deque};
use nb::block;
use common::controller::{DownstreamMessage, UpstreamMessage};
use crate::state::State;

use core::panic::PanicInfo;
use core::sync::atomic::Ordering;
use arduino_hal::{default_serial, Peripherals, pins, Usart};
use arduino_hal::hal::port::{PD2, PD3, PE0, PE1};
use arduino_hal::hal::usart::Event;
use arduino_hal::hal::wdt;
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

static USB_SERIAL: Mutex<RefCell<Option<Usart<USART0, Pin<Input, PE0>, Pin<Output, PE1>>>>> = Mutex::new(RefCell::new(None));
static NANO_SERIAL_RX: Mutex<RefCell<Option<UsartReader<USART1, Pin<Input, PD2>, Pin<Output, PD3>>>>> = Mutex::new(RefCell::new(None));

static mut NANO_QUEUE: Queue<u8, 256> = Queue::new();
static NANO_PRODUCER: Mutex<RefCell<Option<Producer<u8, 256>>>> = Mutex::new(RefCell::new(None));
static NANO_CONSUMER: Mutex<RefCell<Option<Consumer<u8, 256>>>> = Mutex::new(RefCell::new(None));

static mut USB_READ_QUEUE: Queue<u8, 256> = Queue::new();
static USB_READ_PRODUCER: Mutex<RefCell<Option<Producer<u8, 256>>>> = Mutex::new(RefCell::new(None));
static USB_READ_CONSUMER: Mutex<RefCell<Option<Consumer<u8, 256>>>> = Mutex::new(RefCell::new(None));

static mut USB_WRITE_QUEUE: Queue<u8, 256> = Queue::new();
static USB_WRITE_PRODUCER: Mutex<RefCell<Option<Producer<u8, 256>>>> = Mutex::new(RefCell::new(None));
static USB_WRITE_CONSUMER: Mutex<RefCell<Option<Consumer<u8, 256>>>> = Mutex::new(RefCell::new(None));

#[arduino_hal::entry]
fn main() -> ! {
    let mut usb_buffer = Vec::<u8, { mem::size_of::<DownstreamMessage>() + 5 }>::new();
    let mut nano_buffer = Vec::<u8, 32>::new();

    let dp = Peripherals::take().unwrap();
    let pins = pins!(dp);
    let mut usb = default_serial!(dp, pins, common::BAUD_RATE_CTRL);
    let mut nano = Usart::new(dp.USART1, pins.d19, pins.d18.into_output(), common::BAUD_RATE_NANO.into_baudrate());

    usb.listen(Event::RxComplete);
    nano.listen(Event::RxComplete);

    let (nano_reader, _) = nano.split();

    interrupt::free(|cs| {
        USB_SERIAL.borrow(cs).borrow_mut().replace(usb);
        NANO_SERIAL_RX.borrow(cs).borrow_mut().replace(nano_reader);

        let (nano_producer, nano_consumer) = unsafe { NANO_QUEUE.split() };
        NANO_PRODUCER.borrow(cs).replace(Some(nano_producer));
        NANO_CONSUMER.borrow(cs).replace(Some(nano_consumer));

        let (usb_read_producer, usb_read_consumer) = unsafe { USB_READ_QUEUE.split() };
        USB_READ_PRODUCER.borrow(cs).replace(Some(usb_read_producer));
        USB_READ_CONSUMER.borrow(cs).replace(Some(usb_read_consumer));

        let (usb_write_producer, usb_write_consumer) = unsafe { USB_WRITE_QUEUE.split() };
        USB_WRITE_PRODUCER.borrow(cs).replace(Some(usb_write_producer));
        USB_WRITE_CONSUMER.borrow(cs).replace(Some(usb_write_consumer));
    });

    // Enable interrupts globally
    unsafe { interrupt::enable() };

    let mut state = State::default();

    write_message(&UpstreamMessage::Init);

    loop {
        // process data from computer
        loop {
            let byte = interrupt::free(|cs| {
                if let Some(ref mut usb_consumer) = USB_READ_CONSUMER.borrow(cs).borrow_mut().deref_mut() {
                    usb_consumer.dequeue()
                } else {
                    None
                }
            });

            if let Some(byte) = byte {
                if let Ok(()) = usb_buffer.push(byte) {
                    if common::end_of_frame(&byte) {
                        match common::read(&mut usb_buffer) {
                            Ok(message) => {
                                state.update(message);
                                write_message(&UpstreamMessage::Ack);
                            }
                            Err(e) => {
                                // data was corrupted during transmission
                                write_message(&UpstreamMessage::BadP(e));
                            }
                        }
                        usb_buffer.clear();
                    }
                } else {
                    // data buffer was over run, a seperator byte was mis-received
                    write_message(&UpstreamMessage::BadO);
                    usb_buffer.clear();
                }
            } else {
                break;
            }
        }

        // forward data from nano
        while !nano_buffer.is_full() {
            let done = interrupt::free(|cs| {
                if let Some(ref mut nano_consumer) = NANO_CONSUMER.borrow(cs).borrow_mut().deref_mut() {
                    if let Some(byte) = nano_consumer.dequeue() {
                        nano_buffer.push(byte).unwrap();
                        return false;
                    }
                }

                true
            });
            if done {
                break;
            }
        }

        if !nano_buffer.is_empty() {
            write_message(&UpstreamMessage::IMUStream(&nano_buffer));
            nano_buffer.clear();
        }
    }
}

#[avr_device::interrupt(atmega2560)]
#[allow(non_snake_case)]
fn USART0_RX() {
    interrupt::free(|cs| {
        if let Some(ref mut usb) = USB_SERIAL.borrow(cs).borrow_mut().deref_mut() {
            if let Some(ref mut usb_producer) = USB_READ_PRODUCER.borrow(cs).borrow_mut().deref_mut() {
                while let Ok(byte) = usb.read() {
                    //todo remove unwrap
                    let _ = usb_producer.enqueue(byte).unwrap();
                }
            }
        }
    });
}

#[avr_device::interrupt(atmega2560)]
#[allow(non_snake_case)]
fn USART0_UDRE() {
    interrupt::free(|cs| {
        if let Some(ref mut usb) = USB_SERIAL.borrow(cs).borrow_mut().deref_mut() {
            if let Some(ref mut usb_consumer) = USB_WRITE_CONSUMER.borrow(cs).borrow_mut().deref_mut() {
                if let Some(byte) = usb_consumer.dequeue() {
                    let _ = usb.write(byte).unwrap();
                } else {
                    usb.unlisten(Event::DataRegisterEmpty);
                }
            }
        }
    });
}

#[avr_device::interrupt(atmega2560)]
#[allow(non_snake_case)]
fn USART1_RX() {
    interrupt::free(|cs| {
        if let Some(ref mut nano_rx) = NANO_SERIAL_RX.borrow(cs).borrow_mut().deref_mut() {
            if let Some(ref mut nano_producer) = NANO_PRODUCER.borrow(cs).borrow_mut().deref_mut() {
                while let Ok(byte) = nano_rx.read() {
                    //todo remove unwrap
                    let _ = nano_producer.enqueue(byte).unwrap();
                }
            }
        }
    });
}

/// This function is unsafe when called from an interrupt handler
fn write_message(message: &UpstreamMessage) {
    static mut OUT_BUFFER: [u8; 200] = [0; 200];

    let buffer = unsafe { &mut OUT_BUFFER };
    if let Ok(buffer) = common::write(message, buffer) {
        for &mut byte in buffer {
            interrupt::free(|cs| {
                if let Some(ref mut usb_producer) = USB_WRITE_PRODUCER.borrow(cs).borrow_mut().deref_mut() {
                    //todo remove unwrap
                    let _ = usb_producer.enqueue(byte).unwrap();
                }
            });
        }

        interrupt::free(|cs| {
            if let Some(ref mut serial) = USB_SERIAL.borrow(cs).borrow_mut().deref_mut() {
                serial.listen(Event::DataRegisterEmpty);
            }
        });
    }
}
