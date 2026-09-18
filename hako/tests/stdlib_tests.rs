use hako::stdlib::STDLIB;

// ===== Stdlib Tests =====

#[test]
fn test_stdlib_contains_port_io() {
    assert!(STDLIB.contains("pub fn port_outb("));
    assert!(STDLIB.contains("pub fn port_inb("));
}

#[test]
fn test_stdlib_contains_serial() {
    assert!(STDLIB.contains("pub fn serial_config("));
    assert!(STDLIB.contains("pub fn serial_write_byte("));
    assert!(STDLIB.contains("pub fn serial_read_byte("));
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
    // Stdlib should use core::arch::asm for bare metal
    assert!(STDLIB.contains("core::arch::asm!"));
    assert!(STDLIB.contains("core::ptr::write_volatile"));
}

#[test]
fn test_stdlib_serial_config_logic() {
    // Check that serial_config sets up the COM port correctly
    assert!(STDLIB.contains("port_outb(0x80, com + 3)")); // DLAB
    assert!(STDLIB.contains("port_outb(1, com)")); // Low byte
    assert!(STDLIB.contains("port_outb(0, com + 1)")); // High byte
    assert!(STDLIB.contains("port_outb(3, com + 3)")); // 8N1
}

#[test]
fn test_stdlib_vga_clear_logic() {
    // Check that vga_clear uses 0x0720 (space with white on black)
    assert!(STDLIB.contains("0x0720"));
    assert!(STDLIB.contains("80 * 25"));
}

#[test]
fn test_stdlib_pit_config_logic() {
    // Check that pit_config calculates divisor correctly
    assert!(STDLIB.contains("1193180u32 / freq"));
    assert!(STDLIB.contains("port_outb(0x36, 0x43)")); // Command byte
}
