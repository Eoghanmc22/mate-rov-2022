use arduino_hal::Adc;
use arduino_hal::hal::Atmega;
use arduino_hal::port::Pin;
use arduino_hal::port::mode::{Analog, Floating, Input};
use avr_device::atmega2560::ADC;
use avr_hal_generic::adc::AdcChannel;
use avr_hal_generic::port::PinOps;
use common::controller::VelocityData;

pub struct Joystick<Lx, Ly, Rx, Ry>
{
    left_x:  Pin<Analog, Lx>,
    left_y:  Pin<Analog, Ly>,
    right_x: Pin<Analog, Rx>,
    right_y: Pin<Analog, Ry>,
}

impl<Lx, Ly, Rx, Ry> Joystick<Lx, Ly, Rx, Ry>
    where
        Lx: PinOps,
        Ly: PinOps,
        Rx: PinOps,
        Ry: PinOps,
        Pin<Analog, Lx>: AdcChannel<Atmega, ADC>,
        Pin<Analog, Ly>: AdcChannel<Atmega, ADC>,
        Pin<Analog, Rx>: AdcChannel<Atmega, ADC>,
        Pin<Analog, Ry>: AdcChannel<Atmega, ADC>,
{
    pub fn new(lx: Pin<Input<Floating>, Lx>, ly: Pin<Input<Floating>, Ly>, rx: Pin<Input<Floating>, Rx>, ry: Pin<Input<Floating>, Ry>, adc: &mut Adc) -> Self {
        Joystick {
            left_x: lx.into_analog_input(adc),
            left_y: ly.into_analog_input(adc),
            right_x: rx.into_analog_input(adc),
            right_y: ry.into_analog_input(adc)
        }
    }

    pub fn read(&self, adc: &mut Adc) -> VelocityData {
        const ADC_SCALE: f32 = 1023.0;

        let max = 0.95;
        let min = 0.05;

        let lx = self.left_x.analog_read(adc) as f32 / ADC_SCALE * 2.0 - 1.0;
        let ly = self.left_y.analog_read(adc) as f32 / ADC_SCALE * 2.0 - 1.0;
        let rx = self.right_x.analog_read(adc) as f32 / ADC_SCALE * 2.0 - 1.0;
        let ry = self.right_y.analog_read(adc) as f32 / ADC_SCALE * 2.0 - 1.0;

        let lx = common::clamp_map_val(lx, min, max);
        let ly = common::clamp_map_val(ly, min, max);
        let rx = common::clamp_map_val(rx, min, max);
        let ry = common::clamp_map_val(ry, min, max);

        common::joystick_math(lx, ly, rx, ry)
    }
}
