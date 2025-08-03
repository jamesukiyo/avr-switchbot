use avr_device::interrupt::Mutex;
use core::cell::Cell;

// servo positions
pub const SERVO_MIN: u16 = 125; // 0.5ms pulse
pub const SERVO_MAX: u16 = 625; // 2.5ms pulse
pub const SERVO_MID: u16 = 375; // 1.5ms pulse

pub struct Servo {
    pos: Mutex<Cell<u16>>,
}

impl Servo {
    /// new servo instance
    pub const fn new() -> Self {
        Self {
            pos: Mutex::new(Cell::new(SERVO_MIN)),
        }
    }

    /// set servo position
    pub fn set_pos(&self, timer: &arduino_hal::pac::TC1, pos: u16) {
        avr_device::interrupt::free(|cs| self.pos.borrow(cs).set(pos));
        timer.ocr1a.write(|w| w.bits(pos));
    }

    /// get current servo position
    pub fn get_pos(&self) -> u16 {
        avr_device::interrupt::free(|cs| self.pos.borrow(cs).get())
    }

    /// toggle servo between min and max
    pub fn toggle(&self, timer: &arduino_hal::pac::TC1) -> u16 {
        let curr_pos = self.get_pos();
        let new_pos = if curr_pos <= SERVO_MID {
            SERVO_MAX
        } else {
            SERVO_MIN
        };
        self.set_pos(timer, new_pos);
        new_pos
    }
}
