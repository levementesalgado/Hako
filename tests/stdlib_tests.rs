use hako::stdlib::STDLIB;

// ===== Stdlib Tests =====

#[test]
fn test_stdlib_contains_port_io() {
    assert!(STDLIB.contains("pub fn port_outb("));
    assert!(STDLIB.contains("pub fn port_inb("));
    assert!(STDLIB.contains("pub fn port_outw("));
    assert!(STDLIB.contains("pub fn port_inw("));
}

#[test]
fn test_stdlib_contains_serial() {
    assert!(STDLIB.contains("pub fn serial_config("));
    assert!(STDLIB.contains("pub fn serial_write_byte("));
    assert!(STDLIB.contains("pub fn serial_read_byte("));
    assert!(STDLIB.contains("pub fn serial_write_str("));
    assert!(STDLIB.contains("pub fn serial_read_str("));
}

#[test]
fn test_stdlib_contains_uart() {
    assert!(STDLIB.contains("pub fn uart_init("));
    assert!(STDLIB.contains("pub fn uart_write("));
    assert!(STDLIB.contains("pub fn uart_read("));
    assert!(STDLIB.contains("pub fn uart_write_str("));
}

#[test]
fn test_stdlib_contains_spi() {
    assert!(STDLIB.contains("pub fn spi_init("));
    assert!(STDLIB.contains("pub fn spi_transfer_byte("));
    assert!(STDLIB.contains("pub fn spi_write("));
}

#[test]
fn test_stdlib_contains_i2c() {
    assert!(STDLIB.contains("pub fn i2c_init("));
    assert!(STDLIB.contains("pub fn i2c_start("));
    assert!(STDLIB.contains("pub fn i2c_stop("));
    assert!(STDLIB.contains("pub fn i2c_write_bit("));
    assert!(STDLIB.contains("pub fn i2c_read_bit("));
    assert!(STDLIB.contains("pub fn i2c_write_byte("));
}

#[test]
fn test_stdlib_contains_gpio() {
    assert!(STDLIB.contains("pub fn gpio_output("));
    assert!(STDLIB.contains("pub fn gpio_input("));
    assert!(STDLIB.contains("pub fn gpio_write("));
    assert!(STDLIB.contains("pub fn gpio_read("));
}

#[test]
fn test_stdlib_contains_vga() {
    assert!(STDLIB.contains("pub fn vga_put_char("));
    assert!(STDLIB.contains("pub fn vga_read_cursor("));
    assert!(STDLIB.contains("pub fn vga_write_str("));
    assert!(STDLIB.contains("pub fn vga_clear("));
    assert!(STDLIB.contains("pub fn vga_scroll("));
    assert!(STDLIB.contains("pub fn vga_set_cursor("));
}

#[test]
fn test_stdlib_contains_pit() {
    assert!(STDLIB.contains("pub fn pit_config("));
}

#[test]
fn test_stdlib_contains_keyboard() {
    assert!(STDLIB.contains("pub fn keyboard_init("));
}

#[test]
fn test_stdlib_uses_no_std() {
    assert!(STDLIB.contains("core::arch::asm!"));
    assert!(STDLIB.contains("core::ptr::write_volatile"));
}

#[test]
fn test_stdlib_serial_config_logic() {
    assert!(STDLIB.contains("port_outb(0x80, com + 3)"));
    assert!(STDLIB.contains("port_outb(1, com)"));
    assert!(STDLIB.contains("port_outb(0, com + 1)"));
    assert!(STDLIB.contains("port_outb(3, com + 3)"));
}

#[test]
fn test_stdlib_vga_clear_logic() {
    assert!(STDLIB.contains("0x0720"));
    assert!(STDLIB.contains("80 * 25"));
}

#[test]
fn test_stdlib_pit_config_logic() {
    assert!(STDLIB.contains("1193180u32 / freq"));
    assert!(STDLIB.contains("port_outb(0x36, 0x43)"));
}

#[test]
fn test_stdlib_uart_init_logic() {
    assert!(STDLIB.contains("pub fn uart_init("));
    assert!(STDLIB.contains("port_outb(0x80, base + 3)"));
    assert!(STDLIB.contains("port_outb(0x03, base)"));
}

#[test]
fn test_stdlib_spi_bitbang() {
    assert!(STDLIB.contains("pub fn spi_transfer_byte("));
    assert!(STDLIB.contains("gpio_write(mosi,"));
    assert!(STDLIB.contains("gpio_write(sclk,"));
}

#[test]
fn test_stdlib_i2c_bitbang() {
    assert!(STDLIB.contains("pub fn i2c_start("));
    assert!(STDLIB.contains("pub fn i2c_stop("));
    assert!(STDLIB.contains("gpio_write(sda, 0)"));
    assert!(STDLIB.contains("gpio_write(scl, 1)"));
}
