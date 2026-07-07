pub mod linux;

use crate::serial_driver;
use crate::fs;

pub fn find_first_elf() -> Option<&'static [u8]> {
    for file in fs::file_list() {
        if file.ends_with(".elf") {
            serial_driver::sprintln!("  personality: found ELF: {}", file);
            return fs::file_read(&file);
        }
    }
    None
}

pub fn init() {
    if let Some(data) = find_first_elf() {
        serial_driver::sprintln!("  personality: {} bytes", data.len());
        // Old behavior: load and jump directly
        match linux::load_elf(data) {
            Ok((pd, entry, sp)) => {
                serial_driver::sprintln!("  personality: jumping to ring 3 pd={:#x} entry={:#x} sp={:#x}", pd, entry, sp);
                crate::vga_driver::write_str("  personality: running ELF...\n");
                unsafe { linux::jump_to_user_entry(pd, entry, sp); }
            }
            Err(e) => {
                serial_driver::sprintln!("  personality: ELF load error: {}", e);
            }
        }
    } else {
        serial_driver::sprintln!("  personality: no ELF found");
    }
}
