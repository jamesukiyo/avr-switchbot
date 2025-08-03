/*
 * Home-made "switchbot" with an Arduino Uno, Rust and avr-hal.
 *
 * CHQ1838 infrared receiver + SG90 servo control for the Arduino Uno using
 * avr-hal by Rahix: https://github.com/Rahix/avr-hal.
 *
 * This program receives infrared signals from a remote control and controls a
 * servo motor based on any button presses. Currently, any button
 * press will cause the same actions:
 *   1. move servo to maximum position
 *   2. wait 1 second
 *   3. move servo to minimum position
 *
 * Author: James Plummer <jamesp2001@live.co.uk>
 * Repository: https://github.com/jamesukiyo/switchbot
 * License: MIT
 * Last modified: 2025-08-03
 *
 * Hardware:
 *   - Arduino Uno (with the ATmega328P microcontroller)
 *   - CHQ1838 infrared receiver
 *   - SG90 servo motor
 *   - Generic infrared remote control
 *
 * Connections:
 *   - CHQ1838  OUT -> D2  (infrared receiver)
 *   - CHQ1838  VCC -> 5V
 *   - CHQ1838  GND -> GND
 *   - SG90     IN  -> D9  (D9 pwm output)
 *   - SG90     VCC -> 5V
 *   - SG90     GND -> GND
 *
 * Shorthands:
 *   c       counter/cell
 *   cs      critical section
 *   ctc     clear timer (on) compare
 *   exint   external interrupt
 *   icr     input capture register
 *   ocr     output compare register
 *   pcicr   pin change interrupt control register
 *   pcmsk   pin change interrupt mask register
 *   rx      receiver
 *   tc      timer/counter
 *   tccr    timer control register
 *   v       value
 *   w       write register
 */

#![warn(clippy::pedantic)]
#![allow(static_mut_refs)] // unavoidable
#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)] // avr interrupt handling

use panic_halt as _; // halt on panic

use arduino_hal::delay_ms;
use arduino_hal::hal::port::PD2;
use arduino_hal::port::mode::{Floating, Input};
use arduino_hal::port::Pin;
use arduino_hal::prelude::*;

use avr_device::interrupt::Mutex; // interrupt-safe mutex for sharing data between main code and interrupts

use core::cell::Cell; // mutable memory location

use infrared::protocol::nec::NecCommand;
use infrared::protocol::Nec;
use infrared::Receiver;

use ufmt::uwriteln;

mod clock;
mod servo;
use clock::Clock;
use servo::{Servo, SERVO_MAX, SERVO_MIN};

// types for readability
type IrPin = Pin<Input<Floating>, PD2>; // D2 pin as floating input
type IrProto = Nec;
type IrCmd = NecCommand;

// globals shared between main and interrupt handlers
static CLOCK: Clock = Clock::new(); // timer for ir signal timing
static mut RECEIVER: Option<Receiver<IrProto, IrPin>> = None;
static SERVO: Servo = Servo::new();

// thread-safe store for ir commands
static CMD: Mutex<Cell<Option<IrCmd>>> = Mutex::new(Cell::new(None));

// interrupt handler for D2 pin changes
#[avr_device::interrupt(atmega328p)]
fn PCINT2() {
    // get ir receiver and timestamp
    let recv = unsafe { RECEIVER.as_mut().unwrap() };
    let now = CLOCK.now();

    // try to decode ir signal
    if let Ok(Some(cmd)) = recv.event_instant(now) {
        // complete ir command decoded
        avr_device::interrupt::free(|cs| {
            let cell = CMD.borrow(cs);
            cell.set(Some(cmd));
        });
        // ignored:
        // Ok(None) = partial signal
        // Err(_) = decode error
        // could add LED for error indication as seen in infrared example
    }
}

// timer interrupt every 50 microseconds for ir timing
#[avr_device::interrupt(atmega328p)]
fn TIMER0_COMPA() {
    CLOCK.tick(); // increment timing counter
}

#[arduino_hal::entry]
fn main() -> ! {
    // initialise device peripherals, pins and serial
    let dp = arduino_hal::Peripherals::take().unwrap(); // dp = device peripherals
    let pins = arduino_hal::pins!(dp);
    let mut serial = arduino_hal::default_serial!(dp, pins, 57600);

    // start clock for time tracking
    CLOCK.start(&dp.TC0);

    // configure servo pwm on d9
    let _servo_pin = pins.d9.into_output();

    // configure Timer1 for servo pwm (50hz, 20ms period)
    dp.TC1.icr1.write(|w| w.bits(4999));

    // phase + frequency correct pwm
    dp.TC1
        .tccr1a
        .write(|w| w.wgm1().bits(0b10).com1a().match_clear());

    // prescaler 64 gives 250khz (16mhz atmega328p clock / 64 = 250khz)
    dp.TC1
        .tccr1b
        .write(|w| w.wgm1().bits(0b11).cs1().prescale_64());

    // initial servo position
    // ocr1a is connected to D9 on the arduino uno
    dp.TC1.ocr1a.write(|w| w.bits(SERVO_MIN));

    // configure pin change interrupts for ir receiver
    dp.EXINT.pcicr.write(|w| unsafe { w.bits(0b100) });

    // enable interrupt on PCINT18 which is pin PD2
    dp.EXINT.pcmsk2.write(|w| w.bits(0b100));

    // create ir receiver
    let ir = Receiver::with_pin(Clock::FREQ, pins.d2);

    // move ir receiver to global for interrupt access
    unsafe {
        RECEIVER.replace(ir);
    }

    // enable interrupts globally
    unsafe { avr_device::interrupt::enable() };

    // test servo on startup min -> max -> min
    uwriteln!(&mut serial, "Testing servo... MIN -> MAX -> MIN.\r").unwrap_infallible();
    SERVO.set_pos(&dp.TC1, SERVO_MIN);
    uwriteln!(&mut serial, "moved to start ({} counts)\r", SERVO_MIN).unwrap_infallible();
    delay_ms(1000);
    SERVO.set_pos(&dp.TC1, SERVO_MAX);
    uwriteln!(&mut serial, "moved to end ({} counts)\r", SERVO_MAX).unwrap_infallible();
    delay_ms(1000);
    SERVO.set_pos(&dp.TC1, SERVO_MIN);
    uwriteln!(&mut serial, "back to start ({} counts)\r", SERVO_MIN).unwrap_infallible();
    delay_ms(1000);

    uwriteln!(&mut serial, "Startup complete :]\r").unwrap_infallible();

    loop {
        // check for ir commands
        if let Some(cmd) = avr_device::interrupt::free(|cs| CMD.borrow(cs).take()) {
            uwriteln!(
                &mut serial,
                "NEC Cmd: Address: {}, Command: {}, Repeat?: {}\r",
                cmd.addr,
                cmd.cmd,
                cmd.repeat
            )
            .unwrap_infallible();

            // only respond to button presses, not repeats
            if !cmd.repeat && cmd.cmd != 0 {
                // toggle servo between min and max
                let new_pos = SERVO.toggle(&dp.TC1);

                // back to start after 1s
                delay_ms(1000);
                SERVO.set_pos(&dp.TC1, SERVO_MIN);

                uwriteln!(
                    &mut serial,
                    "Servo position: {} counts ({}ms pulse)\r",
                    new_pos,
                    new_pos * 4 // each count is 4 microseconds
                )
                .unwrap_infallible();
            }
        }

        delay_ms(100);
    }
}
