// Hako stdlib source — injected as a module into generated output
pub const STDLIB: &str = r##"
// --- Port I/O ---
pub fn port_outb(value: u32, port: u32) {
    unsafe { core::arch::asm!("out dx, al", in("dx") port as u16, in("al") value as u8); }
}

pub fn port_inb(port: u32) -> u32 {
    let result: u8;
    unsafe { core::arch::asm!("in al, dx", in("dx") port as u16, out("al") result); }
    result as u32
}

pub fn port_outw(value: u32, port: u32) {
    unsafe { core::arch::asm!("out dx, ax", in("dx") port as u16, in("ax") value as u16); }
}

pub fn port_inw(port: u32) -> u32 {
    let result: u16;
    unsafe { core::arch::asm!("in ax, dx", in("dx") port as u16, out("ax") result); }
    result as u32
}

// --- Serial (COM) ---
pub fn serial_config(com: u32) {
    port_outb(0x80, com + 3);
    port_outb(1, com);
    port_outb(0, com + 1);
    port_outb(3, com + 3);
}

pub fn serial_write_byte(b: u32, com: u32) {
    loop {
        if port_inb(com + 5) & 0x20 != 0 { break; }
    }
    port_outb(b, com);
}

pub fn serial_read_byte(com: u32) -> u32 {
    loop {
        if port_inb(com + 5) & 1 != 0 { break; }
    }
    port_inb(com)
}

pub fn serial_write_str(s: &str, com: u32) {
    for c in s.bytes() {
        serial_write_byte(c as u32, com);
    }
}

pub fn serial_read_str(com: u32) -> [u8; 256] {
    let mut buf = [0u8; 256];
    let mut i = 0;
    loop {
        let b = serial_read_byte(com);
        if b == 0 || i >= 255 { break; }
        buf[i] = b as u8;
        i += 1;
    }
    buf
}

// --- UART (16550) ---
pub fn uart_init(base: u32) {
    port_outb(0x00, base + 1);
    port_outb(0x80, base + 3);
    port_outb(0x03, base);
    port_outb(0x00, base + 1);
    port_outb(0x03, base + 3);
    port_outb(0xC7, base + 2);
    port_outb(0x0B, base + 4);
}

pub fn uart_write(base: u32, byte: u32) {
    while port_inb(base + 5) & 0x20 == 0 {}
    port_outb(byte, base);
}

pub fn uart_read(base: u32) -> u32 {
    while port_inb(base + 5) & 1 == 0 {}
    port_inb(base)
}

pub fn uart_write_str(base: u32, s: &str) {
    for c in s.bytes() {
        uart_write(base, c as u32);
    }
}

// --- SPI (bit-bang) ---
pub fn spi_init(mosi: u32, sclk: u32, cs: u32) {
    gpio_output(mosi);
    gpio_output(sclk);
    gpio_output(cs);
    gpio_write(cs, 1);
}

pub fn spi_transfer_byte(mosi: u32, sclk: u32, cs: u32, byte: u32) -> u32 {
    let mut result = 0u32;
    gpio_write(cs, 0);
    let mut i = 8;
    loop {
        if i == 0 { break; }
        i -= 1;
        gpio_write(mosi, (byte >> i) & 1);
        gpio_write(sclk, 1);
        gpio_write(sclk, 0);
        result = (result << 1) | gpio_read(mosi);
    }
    gpio_write(cs, 1);
    result
}

pub fn spi_write(mosi: u32, sclk: u32, cs: u32, data: &[u8]) {
    for &byte in data {
        spi_transfer_byte(mosi, sclk, cs, byte as u32);
    }
}

// --- I2C (bit-bang) ---
pub fn i2c_init(sda: u32, scl: u32) {
    gpio_input(sda);
    gpio_output(scl);
    gpio_write(scl, 1);
}

pub fn i2c_start(sda: u32, scl: u32) {
    gpio_output(sda);
    gpio_write(sda, 1);
    gpio_write(scl, 1);
    gpio_write(sda, 0);
    gpio_write(scl, 0);
}

pub fn i2c_stop(sda: u32, scl: u32) {
    gpio_output(sda);
    gpio_write(scl, 0);
    gpio_write(sda, 0);
    gpio_write(scl, 1);
    gpio_write(sda, 1);
}

pub fn i2c_write_bit(sda: u32, scl: u32, bit: u32) {
    gpio_write(sda, bit);
    gpio_write(scl, 1);
    gpio_write(scl, 0);
}

pub fn i2c_read_bit(sda: u32, scl: u32) -> u32 {
    gpio_input(sda);
    gpio_write(scl, 1);
    let bit = gpio_read(sda);
    gpio_write(scl, 0);
    bit
}

pub fn i2c_write_byte(sda: u32, scl: u32, byte: u32) -> u32 {
    let mut i = 8;
    loop {
        if i == 0 { break; }
        i -= 1;
        i2c_write_bit(sda, scl, (byte >> i) & 1);
    }
    i2c_read_bit(sda, scl)
}

// --- GPIO ---
pub fn gpio_output(pin: u32) {
    // Linux sysfs GPIO
    // In bare-metal: use MMIO to set direction register
}

pub fn gpio_input(pin: u32) {
    // Linux sysfs GPIO
    // In bare-metal: use MMIO to set direction register
}

pub fn gpio_write(pin: u32, value: u32) {
    // Linux sysfs GPIO
    // In bare-metal: use MMIO to set output register
}

pub fn gpio_read(pin: u32) -> u32 {
    // Linux sysfs GPIO
    // In bare-metal: use MMIO to read input register
    0
}

// --- GPIO Pin Macros ---
// These provide named pin access for common boards
pub macro pin($port:expr, $pin:expr) { ($port * 8 + $pin) }
pub macro led_green() { pin!(0, 0) }
pub macro led_red() { pin!(0, 1) }
pub macro led_blue() { pin!(0, 2) }
pub macro btn_s1() { pin!(1, 0) }
pub macro btn_s2() { pin!(1, 1) }
pub macro btn_s3() { pin!(1, 2) }
pub macro spi_mosi() { pin!(2, 3) }
pub macro spi_miso() { pin!(2, 4) }
pub macro spi_sclk() { pin!(2, 5) }
pub macro spi_cs() { pin!(2, 6) }
pub macro i2c_sda() { pin!(3, 0) }
pub macro i2c_scl() { pin!(3, 1) }

// --- VGA text mode ---
pub fn vga_put_char(c: u32, x: u32, y: u32) {
    let pos = x + y * 80;
    let addr = 0xB8000 + pos * 2;
    unsafe {
        core::ptr::write_volatile(addr as *mut u8, c as u8);
        core::ptr::write_volatile((addr + 1) as *mut u8, 0x07);
    }
}

pub fn vga_read_cursor() -> (u32, u32) {
    port_outb(14, 0x3D4);
    let hi = port_inb(0x3D5);
    port_outb(15, 0x3D4);
    let lo = port_inb(0x3D5);
    let pos = (hi << 8) | lo;
    (pos % 80, pos / 80)
}

pub fn vga_write_str(s: &str) {
    let (mut x, mut y) = vga_read_cursor();
    for c in s.bytes() {
        if c == b'\\n' { x = 0; y += 1; continue; }
        vga_put_char(c as u32, x, y);
        x += 1;
        if x >= 80 { x = 0; y += 1; }
    }
    vga_set_cursor(x, y);
}

pub fn vga_clear() {
    unsafe {
        let buf = 0xB8000 as *mut u16;
        for i in 0..(80 * 25) {
            core::ptr::write_volatile(buf.add(i), 0x0720);
        }
    }
    vga_set_cursor(0, 0);
}

pub fn vga_scroll() {
    unsafe {
        core::ptr::copy(0xB8000 as *const u16, 0xB8000 as *mut u16, 80 * 24);
        let buf = 0xB8000 as *mut u16;
        for i in (80 * 24)..(80 * 25) {
            core::ptr::write_volatile(buf.add(i), 0x0720);
        }
    }
}

pub fn vga_set_cursor(x: u32, y: u32) {
    let pos = x + y * 80;
    port_outb(14, 0x3D4);
    port_outb((pos >> 8) as u32, 0x3D5);
    port_outb(15, 0x3D4);
    port_outb(pos as u32, 0x3D5);
}

// --- PIT ---
pub fn pit_config(freq: u32) {
    let divisor = 1193180u32 / freq;
    port_outb(0x36, 0x43);
    port_outb(divisor & 0xFF, 0x40);
    port_outb((divisor >> 8) & 0xFF, 0x40);
}

// --- Keyboard ---
pub fn keyboard_init() {
    port_outb(0xAE, 0x64);
    port_outb(0xF3, 0x60);
    port_outb(0x00, 0x60);
}
"##;
