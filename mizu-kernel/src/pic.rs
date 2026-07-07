use crate::serial_driver;

fn outb(port: u16, val: u8) {
    unsafe { core::arch::asm!("out dx, al", in("dx") port, in("al") val, options(nomem, nostack)); }
}

fn inb(port: u16) -> u8 {
    let val: u8;
    unsafe { core::arch::asm!("in al, dx", out("al") val, in("dx") port, options(nomem, nostack)); }
    val
}

/// PIC remapeamento: IRQ0-7 → INT 0x20-0x27, IRQ8-15 → INT 0x28-0x2F
pub fn init() {
    let mask_master = inb(0x21);
    let mask_slave = inb(0xA1);

    // ICW1: initialize
    outb(0x20, 0x11);
    outb(0xA0, 0x11);

    // ICW2: remap
    outb(0x21, 0x20);  // master → INT 0x20
    outb(0xA1, 0x28);  // slave  → INT 0x28

    // ICW3: cascading
    outb(0x21, 0x04);  // slave at IRQ2
    outb(0xA1, 0x02);  // cascade ID

    // ICW4: 8086 mode
    outb(0x21, 0x01);
    outb(0xA1, 0x01);

    // restore masks (keep everything masked except keyboard and timer)
    outb(0x21, mask_master & !0x03);  // unmask IRQ0 (timer) and IRQ1 (keyboard)
    outb(0xA1, mask_slave);

    serial_driver::sprintln!("  pic: remapped (IRQ0→0x20)");
}

pub fn mask_all() {
    outb(0x21, 0xFF);
    outb(0xA1, 0xFF);
}
