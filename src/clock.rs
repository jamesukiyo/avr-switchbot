use arduino_hal::pac::tc0::tccr0b::CS0_A;
use avr_device::interrupt::Mutex;
use core::cell::Cell;

// clock for ir timing
pub struct Clock {
    cntr: Mutex<Cell<u32>>, // thread-safe counter
}

impl Clock {
    // timing config for ir decoding
    pub const FREQ: u32 = 20_000; // 20khz = 50 microseconds ticks
    pub const PRESCALER: CS0_A = CS0_A::PRESCALE_8; // 16mhz atmega328p clock / 8 = 2mhz
    pub const TOP: u8 = 99; // 0-99 = 100 counts = 50 microseconds

    /// new clock starting at zero
    pub const fn new() -> Clock {
        Clock {
            cntr: Mutex::new(Cell::new(0)),
        }
    }

    /// configure and start hardware timer
    #[allow(clippy::unused_self)]
    pub fn start(&self, tc0: &arduino_hal::pac::TC0) {
        // ctc mode (clear timer on compare)
        tc0.tccr0a.write(|w| w.wgm0().ctc());
        tc0.ocr0a.write(|w| w.bits(Self::TOP)); // reset every 50 microseconds
        tc0.tccr0b.write(|w| w.cs0().variant(Self::PRESCALER)); // prescaler

        // enable timer interrupt
        tc0.timsk0.write(|w| w.ocie0a().set_bit());
    }

    /// get current timestamp
    pub fn now(&self) -> u32 {
        avr_device::interrupt::free(|cs| self.cntr.borrow(cs).get())
    }

    /// increment timing cntr
    pub fn tick(&self) {
        avr_device::interrupt::free(|cs| {
            let c = self.cntr.borrow(cs);
            let v = c.get();
            c.set(v.wrapping_add(1)); // prevent overflow
        });
    }
}
