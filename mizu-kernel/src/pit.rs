use crate::serial_driver;
use core::sync::atomic::{AtomicU32, Ordering};

static TICKS: AtomicU32 = AtomicU32::new(0);

/// PIT init: 100 Hz (divisor = 1193180 / 100 = 11932)
pub fn init() {
    let divisor: u16 = 11932;
    unsafe {
        core::arch::asm!(
            "out dx, al",
            in("dx") 0x43u16,
            in("al") 0x36u8,      // channel 0, rate generator, LSB/MSB
            options(nomem, nostack),
        );
        core::arch::asm!(
            "out dx, al",
            in("dx") 0x40u16,
            in("al") (divisor & 0xFF) as u8,
            options(nomem, nostack),
        );
        core::arch::asm!(
            "out dx, al",
            in("dx") 0x40u16,
            in("al") (divisor >> 8) as u8,
            options(nomem, nostack),
        );
    }
    serial_driver::sprintln!("  pit: 100 Hz");
}

pub fn tick() {
    TICKS.fetch_add(1, Ordering::Relaxed);
}

pub fn get_ticks() -> u32 {
    TICKS.load(Ordering::Relaxed)
}
