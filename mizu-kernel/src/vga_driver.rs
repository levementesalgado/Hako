use core::fmt;
use crate::serial_driver;

const VGA_BUF: *mut u8 = 0xB8000 as *mut u8;
const WIDTH: usize = 80;
const HEIGHT: usize = 25;
const TAB: usize = 4;

#[repr(u8)]
pub enum Color {
    Black        = 0,
    Blue         = 1,
    Green        = 2,
    Cyan         = 3,
    Red          = 4,
    Magenta      = 5,
    Brown        = 6,
    LightGray    = 7,
    DarkGray     = 8,
    LightBlue    = 9,
    LightGreen   = 10,
    LightCyan    = 11,
    LightRed     = 12,
    Pink         = 13,
    Yellow       = 14,
    White        = 15,
}

const fn attr(fg: Color, bg: Color) -> u8 {
    (bg as u8) << 4 | (fg as u8)
}

struct Writer {
    row: usize,
    col: usize,
    color: u8,
}

const WRITER_DEFAULT_COLOR: u8 = attr(Color::LightGray, Color::Black);
static mut WRITER: Writer = Writer { row: 0, col: 0, color: WRITER_DEFAULT_COLOR };

#[allow(static_mut_refs)]
fn with_writer(f: impl FnOnce(&mut Writer)) {
    unsafe { f(&mut WRITER) }
}

pub fn set_color(fg: Color, bg: Color) {
    with_writer(|w| w.color = attr(fg, bg));
}

pub fn clear() {
    with_writer(|w| {
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                let offset = (y * WIDTH + x) * 2;
                unsafe {
                    VGA_BUF.add(offset).write(b' ');
                    VGA_BUF.add(offset + 1).write(w.color);
                }
            }
        }
        w.row = 0;
        w.col = 0;
    });
    set_cursor(0, 0);
}

fn scroll(w: &mut Writer) {
    let row_size = WIDTH * 2;
    for y in 1..HEIGHT {
        unsafe {
            core::ptr::copy(
                VGA_BUF.add(y * row_size),
                VGA_BUF.add((y - 1) * row_size),
                row_size,
            );
        }
    }
    let last_row = (HEIGHT - 1) * row_size;
    for x in 0..WIDTH {
        unsafe {
            VGA_BUF.add(last_row + x * 2).write(b' ');
            VGA_BUF.add(last_row + x * 2 + 1).write(w.color);
        }
    }
}

pub(crate) fn put_char(c: u8) {
    if c == b'\n' {
        serial_driver::put_char(b'\r');
    }
    serial_driver::put_char(c);
    with_writer(|w| {
        match c {
            b'\n' => {
                w.col = 0;
                if w.row + 1 < HEIGHT {
                    w.row += 1;
                } else {
                    scroll(w);
                }
            }
            b'\r' => w.col = 0,
            b'\t' => {
                let n = TAB - (w.col % TAB);
                for _ in 0..n {
                    put_char(b' ');
                }
            }
            0x08 => {
                if w.col > 0 {
                    w.col -= 1;
                    let offset = (w.row * WIDTH + w.col) * 2;
                    unsafe {
                        VGA_BUF.add(offset).write(b' ');
                        VGA_BUF.add(offset + 1).write(w.color);
                    }
                }
            }
            c if c >= b' ' && c <= 0x7E => {
                let offset = (w.row * WIDTH + w.col) * 2;
                unsafe {
                    VGA_BUF.add(offset).write(c);
                    VGA_BUF.add(offset + 1).write(w.color);
                }
                w.col += 1;
                if w.col >= WIDTH {
                    w.col = 0;
                    if w.row + 1 < HEIGHT {
                        w.row += 1;
                    } else {
                        scroll(w);
                    }
                }
            }
            _ => {}
        }
    });
    update_cursor_pos();
}

fn update_cursor_pos() {
    with_writer(|w| {
        let pos = (w.row * WIDTH + w.col) as u16;
        unsafe {
            core::arch::asm!("out dx, al", in("dx") 0x3D4u16, in("al") 0x0Fu8, options(nomem, nostack));
            core::arch::asm!("out dx, al", in("dx") 0x3D5u16, in("al") (pos & 0xFF) as u8, options(nomem, nostack));
            core::arch::asm!("out dx, al", in("dx") 0x3D4u16, in("al") 0x0Eu8, options(nomem, nostack));
            core::arch::asm!("out dx, al", in("dx") 0x3D5u16, in("al") ((pos >> 8) & 0xFF) as u8, options(nomem, nostack));
        }
    });
}

pub fn set_cursor(row: usize, col: usize) {
    let pos = (row * WIDTH + col) as u16;
    unsafe {
        core::arch::asm!("out dx, al", in("dx") 0x3D4u16, in("al") 0x0Fu8, options(nomem, nostack));
        core::arch::asm!("out dx, al", in("dx") 0x3D5u16, in("al") (pos & 0xFF) as u8, options(nomem, nostack));
        core::arch::asm!("out dx, al", in("dx") 0x3D4u16, in("al") 0x0Eu8, options(nomem, nostack));
        core::arch::asm!("out dx, al", in("dx") 0x3D5u16, in("al") ((pos >> 8) & 0xFF) as u8, options(nomem, nostack));
    }
}

pub fn move_cursor_left() {
    with_writer(|w| {
        if w.col > 0 {
            w.col -= 1;
        }
    });
    update_cursor_pos();
}

pub fn move_cursor_right() {
    with_writer(|w| {
        if w.col < WIDTH - 1 {
            w.col += 1;
        }
    });
    update_cursor_pos();
}

pub fn set_cursor_visible(visible: bool) {
    unsafe {
        core::arch::asm!("out dx, al", in("dx") 0x3D4u16, in("al") 0x0Au8, options(nomem, nostack));
        let mut cursor = 0u8;
        core::arch::asm!("in al, dx", out("al") cursor, in("dx") 0x3D5u16, options(nomem, nostack));
        if visible {
            cursor &= 0xC0;
            cursor |= 0x0E;
        } else {
            cursor |= 0x20;
        }
        core::arch::asm!("out dx, al", in("dx") 0x3D5u16, in("al") cursor, options(nomem, nostack));
    }
}

pub fn write_char_rc(row: usize, col: usize, c: u8, attr: u8) {
    if row >= HEIGHT || col >= WIDTH { return; }
    let offset = (row * WIDTH + col) * 2;
    unsafe {
        VGA_BUF.add(offset).write(c);
        VGA_BUF.add(offset + 1).write(attr);
    }
}

pub fn write_str_rc(row: usize, col: usize, s: &str, attr: u8) {
    for (i, &b) in s.as_bytes().iter().enumerate() {
        if col + i >= WIDTH { break; }
        write_char_rc(row, col + i, b, attr);
    }
}

pub fn default_attr() -> u8 {
    WRITER_DEFAULT_COLOR
}

pub fn vga_width() -> usize { WIDTH }
pub fn vga_height() -> usize { HEIGHT }

pub fn write_str(s: &str) {
    for b in s.bytes() {
        put_char(b);
    }
}

struct VgaWriter;

impl fmt::Write for VgaWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        write_str(s);
        Ok(())
    }
}

pub fn print(args: fmt::Arguments) {
    use fmt::Write;
    VgaWriter.write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! kprintln {
    () => { $crate::vga_driver::print(format_args!("\n")) };
    ($($arg:tt)*) => {{
        $crate::vga_driver::print(format_args!("{}\n", format_args!($($arg)*)));
    }};
}
