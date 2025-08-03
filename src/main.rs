#![no_std]
#![no_main]

use panic_halt as _;

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    let mut builtin_led = pins.d13.into_output();

    loop {
        builtin_led.toggle();
        arduino_hal::delay_ms(1000);
    }
}
