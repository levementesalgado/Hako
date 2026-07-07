fn inb(port: u16) -> u8 {
    let val: u8;
    unsafe { core::arch::asm!("in al, dx", out("al") val, in("dx") port, options(nomem, nostack)); }
    val
}

static SCAN_NORMAL: [u8; 128] = [
    0,   0,   b'1', b'2', b'3', b'4', b'5', b'6',
    b'7', b'8', b'9', b'0', b'-', b'=', 0,   0,
    b'q', b'w', b'e', b'r', b't', b'y', b'u', b'i',
    b'o', b'p', b'[', b']', b'\n',0,   b'a', b's',
    b'd', b'f', b'g', b'h', b'j', b'k', b'l', b';',
    b'\'',b'`', 0,   b'\\',b'z', b'x', b'c', b'v',
    b'b', b'n', b'm', b',', b'.', b'/', 0,   b'*',
    0,   b' ', 0,   0,   0,   0,   0,   0,
    0,   0,   0,   0,   0,   0,   0,   0,
    0,   0,   0,   0,   0,   0,   0,   0,
    0,   0,   0,   0,   0,   0,   0,   0,
    0,   0,   0,   0,   0,   0,   0,   0,
    0,   0,   0,   0,   0,   0,   0,   0,
    0,   0,   0,   0,   0,   0,   0,   0,
    0,   0,   0,   0,   0,   0,   0,   0,
    0,   0,   0,   0,   0,   0,   0,   0,
];

static SCAN_SHIFT: [u8; 128] = [
    0,   0,   b'!', b'@', b'#', b'$', b'%', b'^',
    b'&', b'*', b'(', b')', b'_', b'+', 0,   0,
    b'Q', b'W', b'E', b'R', b'T', b'Y', b'U', b'I',
    b'O', b'P', b'{', b'}', b'\n',0,   b'A', b'S',
    b'D', b'F', b'G', b'H', b'J', b'K', b'L', b':',
    b'"', b'~', 0,   b'|', b'Z', b'X', b'C', b'V',
    b'B', b'N', b'M', b'<', b'>', b'?', 0,   b'*',
    0,   b' ', 0,   0,   0,   0,   0,   0,
    0,   0,   0,   0,   0,   0,   0,   0,
    0,   0,   0,   0,   0,   0,   0,   0,
    0,   0,   0,   0,   0,   0,   0,   0,
    0,   0,   0,   0,   0,   0,   0,   0,
    0,   0,   0,   0,   0,   0,   0,   0,
    0,   0,   0,   0,   0,   0,   0,   0,
    0,   0,   0,   0,   0,   0,   0,   0,
    0,   0,   0,   0,   0,   0,   0,   0,
];

const KEYBUF_SIZE: usize = 128;
static mut KEYBUF: [u8; KEYBUF_SIZE] = [0; KEYBUF_SIZE];
static mut KEYBUF_RP: usize = 0;
static mut KEYBUF_WP: usize = 0;
static mut SHIFT: bool = false;
static mut CTRL: bool = false;
static mut CAPS: bool = false;
static mut E0_PENDING: bool = false;

pub const KEY_LEFT: u8 = 0x9B;
pub const KEY_RIGHT: u8 = 0x9D;
pub const KEY_UP: u8 = 0x98;
pub const KEY_DOWN: u8 = 0x92;
pub const KEY_HOME: u8 = 0x96;
pub const KEY_END: u8 = 0x97;
pub const KEY_DEL: u8 = 0x9E;

pub fn init() {
    crate::kbd::setup();
}

fn push_key(c: u8) {
    unsafe {
        let next = (KEYBUF_WP + 1) % KEYBUF_SIZE;
        if next != KEYBUF_RP {
            KEYBUF[KEYBUF_WP] = c;
            KEYBUF_WP = next;
        }
    }
}

pub fn pop_key() -> Option<u8> {
    unsafe {
        if KEYBUF_RP == KEYBUF_WP {
            None
        } else {
            let c = KEYBUF[KEYBUF_RP];
            KEYBUF_RP = (KEYBUF_RP + 1) % KEYBUF_SIZE;
            Some(c)
        }
    }
}

pub fn irq_handler() {
    let status = inb(0x64);
    if status & 0x01 == 0 {
        return;
    }
    let scancode = inb(0x60);

    // If E0 was pending from last IRQ, this byte is the extended scancode
    if unsafe { E0_PENDING } {
        unsafe { E0_PENDING = false; }
        if scancode & 0x80 != 0 { return; }
        let k = match scancode {
            0x48 => KEY_UP,
            0x50 => KEY_DOWN,
            0x4B => KEY_LEFT,
            0x4D => KEY_RIGHT,
            0x47 => KEY_HOME,
            0x4F => KEY_END,
            0x53 => KEY_DEL,
            _ => 0,
        };
        if k != 0 { push_key(k); }
        return;
    }

    if scancode & 0x80 != 0 {
        let make = scancode & 0x7F;
        match make {
            0x2A | 0x36 => unsafe { SHIFT = false; },
            0x1D => unsafe { CTRL = false; },
            _ => {}
        }
        return;
    }

    if scancode == 0xE0 {
        unsafe { E0_PENDING = true; }
        return;
    }

    match scancode {
        0x2A | 0x36 => unsafe { SHIFT = true; },
        0x1D => unsafe { CTRL = true; },
        0x3A => unsafe { CAPS = !CAPS; },
        0x0E => push_key(0x08),
        _ => {
            let shift = unsafe { SHIFT };
            let ctrl = unsafe { CTRL };
            let caps = unsafe { CAPS };
            if ctrl {
                let c = SCAN_NORMAL[scancode as usize];
                if c >= b'a' && c <= b'z' { push_key(c - 0x60); }
                else if c >= b'A' && c <= b'Z' { push_key(c - 0x40); }
                return;
            }
            let table = if shift { &SCAN_SHIFT } else { &SCAN_NORMAL };
            if (scancode as usize) >= table.len() { return; }
            let c = table[scancode as usize];
            if c == 0 { return; }
            let out = if caps && c >= b'a' && c <= b'z' { c ^ 0x20 } else { c };
            push_key(out);
        }
    }
}

pub fn get_key() -> Option<u8> {
    if let Some(c) = pop_key() {
        return Some(c);
    }
    let status = inb(0x64);
    if status & 0x01 != 0 {
        let scancode = inb(0x60);

        // If E0 was pending from last poll, this byte is the extended scancode
        if unsafe { E0_PENDING } {
            unsafe { E0_PENDING = false; }
            if scancode & 0x80 != 0 { return None; }
            let k = match scancode {
                0x48 => return Some(KEY_UP),
                0x50 => return Some(KEY_DOWN),
                0x4B => return Some(KEY_LEFT),
                0x4D => return Some(KEY_RIGHT),
                0x47 => return Some(KEY_HOME),
                0x4F => return Some(KEY_END),
                0x53 => return Some(KEY_DEL),
                _ => return None,
            };
        }

        if scancode & 0x80 != 0 {
            let make = scancode & 0x7F;
            match make {
                0x2A | 0x36 => unsafe { SHIFT = false; },
                0x1D => unsafe { CTRL = false; },
                _ => {}
            }
            return None;
        }

        if scancode == 0xE0 {
            unsafe { E0_PENDING = true; }
            return None;
        }

        match scancode {
            0x2A | 0x36 => { unsafe { SHIFT = true; }; None }
            0x1D => { unsafe { CTRL = true; }; None }
            0x3A => { unsafe { CAPS = !CAPS; }; None }
            0x0E => Some(0x08),
            _ => {
                let ctrl = unsafe { CTRL };
                if ctrl {
                    let c = SCAN_NORMAL[scancode as usize];
                    if c >= b'a' && c <= b'z' { return Some(c - 0x60); }
                    if c >= b'A' && c <= b'Z' { return Some(c - 0x40); }
                }
                let shift = unsafe { SHIFT };
                let caps = unsafe { CAPS };
                let table = if shift { &SCAN_SHIFT } else { &SCAN_NORMAL };
                if (scancode as usize) >= table.len() { return None; }
                let c = table[scancode as usize];
                if c == 0 { return None; }
                let out = if caps && c >= b'a' && c <= b'z' { c ^ 0x20 } else { c };
                Some(out)
            }
        }
    } else {
        None
    }
}

pub fn poll_key() -> u8 {
    loop {
        if let Some(c) = get_key() {
            return c;
        }
    }
}
