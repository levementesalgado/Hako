use core::fmt;

const COM1: u16 = 0x3F8;

fn port_in(port: u16) -> u8 {
    let val: u8;
    unsafe { core::arch::asm!("in al, dx", out("al") val, in("dx") port, options(nomem, nostack)); }
    val
}

fn port_out(port: u16, val: u8) {
    unsafe { core::arch::asm!("out dx, al", in("dx") port, in("al") val, options(nomem, nostack)); }
}

pub fn init() {
    port_out(COM1 + 1, 0x00);
    port_out(COM1 + 3, 0x80);
    port_out(COM1 + 0, 0x03);
    port_out(COM1 + 1, 0x00);
    port_out(COM1 + 3, 0x03);
    port_out(COM1 + 2, 0xC7);
    port_out(COM1 + 4, 0x0B);
    port_out(COM1 + 4, 0x0E);
}

fn is_transmit_empty() -> bool {
    port_in(COM1 + 5) & 0x20 != 0
}

pub fn put_char(c: u8) {
    for _ in 0..10000 {
        if is_transmit_empty() {
            port_out(COM1, c);
            return;
        }
    }
}

pub fn get_char() -> Option<u8> {
    if port_in(COM1 + 5) & 1 != 0 {
        Some(port_in(COM1))
    } else {
        None
    }
}

pub fn write_str(s: &str) {
    for b in s.bytes() {
        put_char(b);
    }
}

struct SerialWriter;

impl fmt::Write for SerialWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        write_str(s);
        Ok(())
    }
}

pub fn print(args: fmt::Arguments) {
    use fmt::Write;
    SerialWriter.write_fmt(args).unwrap();
}

macro_rules! sprintln {
    () => { $crate::serial_driver::print(format_args!("\n")) };
    ($($arg:tt)*) => {{
        $crate::serial_driver::print(format_args!("{}\n", format_args!($($arg)*)));
    }};
}

pub(crate) use sprintln;
