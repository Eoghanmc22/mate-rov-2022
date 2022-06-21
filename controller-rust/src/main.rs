#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

mod time;
mod state;
mod sabertooth;
mod spsc;
mod joystick;

use core::cell::RefCell;
use core::mem;
use core::ops::DerefMut;
use arduino_hal::prelude::*;
use embedded_hal::prelude::*;
use embedded_hal::serial::Write;
use heapless::Vec;
use nb::block;
use common::controller::{DownstreamMessage, UpstreamMessage, VelocityData};
use crate::state::State;

use core::panic::PanicInfo;
use core::sync::atomic;
use core::sync::atomic::Ordering;
use arduino_hal::{Adc, default_serial, delay_ms, Peripherals, pins, Usart};
use arduino_hal::hal::port::{PE0, PE1};
use arduino_hal::hal::usart::Event;
use arduino_hal::hal::wdt;
use arduino_hal::port::mode::{Input, Output};
use arduino_hal::port::Pin;
use arduino_hal::usart::UsartReader;
use avr_device::atmega2560::USART0;
use avr_device::interrupt;
use avr_device::interrupt::Mutex;
use spsc::{Consumer, Producer, Queue};
use ufmt::uwriteln;
use common::CommunicationError;
use crate::joystick::Joystick;

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

// Pins
// Joystick: a1, a0, a3, a2
// Sabertooth serial: none (rx), d18 (tx)

// Active low
// Sabertooth e-stop 1: d8
// Sabertooth e-stop 2: d9
// E-stop button: d22
// Joystick-enable: d30

#[arduino_hal::entry]
fn main() -> ! {
    // This buffer will hold partially received packets
    let mut usb_buffer = Vec::<u8, { mem::size_of::<DownstreamMessage>() + 5 }>::new();


    // Setup up peripherals
    let dp = Peripherals::take().unwrap();
    let pins = pins!(dp);

    // Emergency stop
    let mut estop_out_a = pins.d8.into_output_high();
    let mut estop_out_b = pins.d9.into_output_high();
    let estop_in = pins.d22.into_pull_up_input();

    // Joysticks
    let mut adc = Adc::new(dp.ADC, Default::default());
    let joystick = Joystick::new(pins.a1, pins.a0, pins.a3, pins.a2, &mut adc);
    let joystick_enable = pins.d30.into_pull_up_input();

    // Setup Serial
    let mut usb = default_serial!(dp, pins, common::BAUD_RATE_CTRL);
    let mut sabertooth = Usart::new(dp.USART1, pins.d19, pins.d18.into_output(), common::BAUD_RATE_SABERTOOTH.into_baudrate());
    let mut usb_writer = {
        // To improve reliability, we need to handle serial data as soon as it is received
        usb.listen(Event::RxComplete);

        // Split usb interface so the read component can be shared safely
        let (usb_reader, usb_writer) = usb.split();

        // Initialize globals
        interrupt::free(|cs| {
            USB_READER_SERIAL.borrow(cs).borrow_mut().replace(usb_reader);

            let (usb_read_producer, usb_read_consumer) = unsafe { USB_READ_QUEUE.split() };
            USB_READ_PRODUCER.borrow(cs).replace(Some(usb_read_producer));
            USB_READ_CONSUMER.borrow(cs).replace(Some(usb_read_consumer));

            atomic::compiler_fence(Ordering::SeqCst);
        });

        usb_writer
    };


    // Start clock
    time::millis_init(dp.TC0);

    // Wait for sabertooth motor controllers to power on and then initialize it
    delay_ms(2000);
    write_callback(sabertooth::write_init, &mut sabertooth);

    // Enable interrupts globally
    unsafe { interrupt::enable() };

    // Notify the connected pc that we are ready to receive data
    write_message(&UpstreamMessage::Init, &mut usb_writer);

    let mut watchdog = wdt::Wdt::new(dp.WDT, &dp.CPU.mcusr);
    watchdog.start(wdt::Timeout::Ms64).unwrap();

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
                                state.update_pc(message);
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

        // Update joystick state
        {
            if joystick_enable.is_low() {
                let joystick_velocity = joystick.read(&mut adc);
                state.update_joystick(joystick_velocity);
            } else {
                state.update_joystick(VelocityData::default());
            }
        }

        // Read emergency stop button
        {
            let emergency_stop = estop_in.is_high();
            state.update_emergency_stop(emergency_stop);
        }

        // Tell the motor controllers to go into an emergency stop if necessary
        {
            if state.emergency_stop() {
                // The sabertooth's emergency stop pin is active low
                estop_out_a.set_low();
                estop_out_b.set_low();
            }

            // Notify the connected pc
            write_message(&UpstreamMessage::EStop(state.emergency_stop()), &mut usb_writer);
        }
        
        // Respond to pings
        {
            if state.do_ping() {
                write_message(&UpstreamMessage::Pong, &mut usb_writer);
                state.clear_ping();
            }
        }

        // Send updated motor speeds
        {
            let total_velocity = state.compute_velocity();
            let VelocityData { forwards_left, forwards_right, strafing, vertical } = total_velocity;
            write_callback(|buffer| sabertooth::write_speed(buffer, sabertooth::MOTOR_LEFT,     (forwards_left * 127.0) as i8),  &mut sabertooth);
            write_callback(|buffer| sabertooth::write_speed(buffer, sabertooth::MOTOR_RIGHT,    (forwards_right * 127.0) as i8), &mut sabertooth);
            write_callback(|buffer| sabertooth::write_speed(buffer, sabertooth::MOTOR_STRAFING, (strafing * 127.0) as i8),       &mut sabertooth);
            write_callback(|buffer| sabertooth::write_speed(buffer, sabertooth::MOTOR_VERTICAL, (vertical * 127.0) as i8),       &mut sabertooth);

            // Notify the connected pc
            write_message(&UpstreamMessage::TotalVelocity(total_velocity), &mut usb_writer);
        }

        watchdog.feed();
    }
}


// ------------------------
// |      Interrupts      |
// ------------------------

#[avr_device::interrupt(atmega2560)]
#[allow(non_snake_case)]
fn USART0_RX() {
    interrupt::free(|cs| {
        // Access globals
        if let Some(ref mut usb) = USB_READER_SERIAL.borrow(cs).borrow_mut().deref_mut() {
            if let Some(ref mut usb_producer) = USB_READ_PRODUCER.borrow(cs).borrow_mut().deref_mut() {
                // Read all available data
                while let Ok(byte) = usb.read() {
                    let _ = usb_producer.enqueue(byte);
                }
            }
        }
    });
}


// -------------------------
// |     Communication     |
// -------------------------

static mut OUT_BUFFER: [u8; 200] = [0; 200];

/// This function is unsafe when called from an interrupt handler
fn write_message(message: &UpstreamMessage, serial: &mut impl Write<u8>) {
    // Retrieve a temporary buffer and encode the packet into it
    let buffer = unsafe { &mut OUT_BUFFER };
    if let Ok(buffer) = common::write(message, buffer) {
        // Write the buffer
        write_buffer(buffer, serial);
    }
}

/// This function is unsafe when called from an interrupt handler
fn write_callback<F: Fn(&mut [u8]) -> Result<&mut [u8], CommunicationError>>(message_producer: F, serial: &mut impl Write<u8>) {
    // Retrieve a temporary buffer and encode the packet into it
    let buffer = unsafe { &mut OUT_BUFFER };
    if let Ok(buffer) = (message_producer)(buffer) {
        // Write the buffer
        write_buffer(buffer, serial);
    }
}

fn write_buffer(buffer: &[u8], serial: &mut impl Write<u8>) {
    // write each byte to serial
    for &byte in buffer {
        let _ = block!(serial.write(byte));
    }
}
