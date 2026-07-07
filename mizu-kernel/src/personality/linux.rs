use crate::arch::i686::paging::PageDir;

const PAGE_SIZE: usize = 4096;
const STACK_SIZE: usize = 16 * 4096;
const STACK_TOP: u32 = 0xC0000000;

const AT_NULL: u32 = 0;
const AT_PHDR: u32 = 3;
const AT_PHENT: u32 = 4;
const AT_PHNUM: u32 = 5;
const AT_PAGESZ: u32 = 6;
const AT_BASE: u32 = 7;
const AT_ENTRY: u32 = 9;
const AT_RANDOM: u32 = 25;

const PT_LOAD: u32 = 1;
const ELF_MAGIC: [u8; 4] = [0x7F, b'E', b'L', b'F'];

#[derive(Debug)]
pub enum ElfError {
    BadMagic,
    Not32Bit,
    NotExec,
    NoSegments,
    LoadError,
    StackError,
    PagingError,
    BadPhdr,
}

use core::fmt;

impl fmt::Display for ElfError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ElfError::BadMagic => write!(f, "bad ELF magic"),
            ElfError::Not32Bit => write!(f, "not a 32-bit ELF"),
            ElfError::NotExec => write!(f, "not an executable"),
            ElfError::NoSegments => write!(f, "no loadable segments"),
            ElfError::LoadError => write!(f, "segment load failed"),
            ElfError::StackError => write!(f, "stack allocation failed"),
            ElfError::PagingError => write!(f, "page table setup failed"),
            ElfError::BadPhdr => write!(f, "bad program headers"),
        }
    }
}

#[repr(C)]
struct Elf32Header {
    e_ident: [u8; 16],
    e_type: u16,
    e_machine: u16,
    e_version: u32,
    e_entry: u32,
    e_phoff: u32,
    e_shoff: u32,
    e_flags: u32,
    e_ehsize: u16,
    e_phentsize: u16,
    e_phnum: u16,
    e_shentsize: u16,
    e_shnum: u16,
    e_shstrndx: u16,
}

#[repr(C)]
struct Elf32Phdr {
    p_type: u32,
    p_offset: u32,
    p_vaddr: u32,
    p_paddr: u32,
    p_filesz: u32,
    p_memsz: u32,
    p_flags: u32,
    p_align: u32,
}

unsafe fn push_u32(sp: &mut u32, val: u32) {
    *sp -= 4;
    let ptr = *sp as *mut u32;
    ptr.write(val);
}

pub fn load_elf(data: &[u8]) -> Result<(u32, u32, u32), ElfError> {
    if data.len() < core::mem::size_of::<Elf32Header>() {
        return Err(ElfError::BadMagic);
    }

    let hdr: &Elf32Header = unsafe { &*(data.as_ptr() as *const Elf32Header) };

    if hdr.e_ident[0..4] != ELF_MAGIC {
        return Err(ElfError::BadMagic);
    }
    if hdr.e_ident[4] != 1 {
        return Err(ElfError::Not32Bit);
    }
    if hdr.e_type != 2 {
        return Err(ElfError::NotExec);
    }

    let phoff = hdr.e_phoff as usize;
    let phentsize = hdr.e_phentsize as usize;
    let phnum = hdr.e_phnum as usize;

    if phnum == 0 {
        return Err(ElfError::NoSegments);
    }

    let mut pd = unsafe { PageDir::new().ok_or(ElfError::PagingError)? };
    unsafe { pd.identity_map_kernel(); }

    let mut phdr_addr = 0u32;

    // Switch to ELF's PD temporarily to map + copy into its address space
    let saved_pd: u32;
    unsafe {
        core::arch::asm!("mov {}, cr3", out(reg) saved_pd);
        core::arch::asm!("mov cr3, {pd}", pd = in(reg) pd.phys_addr());
    }

    // Helper macro to restore PD before early return
    macro_rules! restore_and_return {
        ($e:expr) => {{
            unsafe { core::arch::asm!("mov cr3, {pd}", pd = in(reg) saved_pd); }
            return $e;
        }};
    }

    for i in 0..phnum {
        let phdr_off = phoff + i * phentsize;
        if phdr_off + core::mem::size_of::<Elf32Phdr>() > data.len() {
            continue;
        }
        let phdr: &Elf32Phdr = unsafe { &*(data.as_ptr().add(phdr_off) as *const Elf32Phdr) };

        if phdr.p_type == PT_LOAD {
            let vaddr = phdr.p_vaddr as usize;
            let filesz = phdr.p_filesz as usize;
            let memsz = phdr.p_memsz as usize;
            let offset = phdr.p_offset as usize;

            let page_start = vaddr & !0xFFF;
            let page_end = (vaddr + memsz + 0xFFF) & !0xFFF;

            // Allocate and map pages for this segment
            let mut page = page_start;
            while page < page_end {
                match unsafe { pd.map_alloc(page, true) } {
                    Some(_) => {}
                    None => {
                        crate::serial_driver::sprintln!("  elf: OOM at vaddr={:#x}", page);
                        restore_and_return!(Err(ElfError::LoadError));
                    }
                }
                page += 4096;
            }

            // Copy file data (vaddr is valid in ELF's PD)
            let avail = data.len().saturating_sub(offset);
            let copy_len = core::cmp::min(filesz, avail);
            if copy_len > 0 {
                unsafe {
                    core::ptr::copy_nonoverlapping(
                        data.as_ptr().add(offset),
                        vaddr as *mut u8,
                        copy_len,
                    );
                }
            }

            // Zero BSS
            let bss_start = vaddr + filesz;
            let bss_end = vaddr + memsz;
            if bss_end > bss_start {
                unsafe {
                    core::ptr::write_bytes(bss_start as *mut u8, 0, bss_end - bss_start);
                }
            }

            let phdr_off_abs = hdr.e_phoff;
            if phdr.p_offset <= phdr_off_abs && phdr_off_abs < phdr.p_offset + phdr.p_filesz {
                phdr_addr = phdr_off_abs - phdr.p_offset + phdr.p_vaddr;
            }
        }
    }

    // Allocate user stack (also in ELF's PD)
    if !unsafe { pd.alloc_stack(STACK_TOP as usize, STACK_SIZE) } {
        restore_and_return!(Err(ElfError::StackError));
    }

    // Setup initial stack with argv, envp, aux vectors
    // Stack grows down. We'll write from the top down.
    let mut sp = STACK_TOP;

    // Step 1: push a filename string
    let fname = b"hello.elf\0";
    let fname_addr = sp - fname.len() as u32;
    unsafe {
        core::ptr::copy_nonoverlapping(fname.as_ptr(), fname_addr as *mut u8, fname.len());
    }
    sp = fname_addr;

    // Step 2: push AT_RANDOM bytes (16 random-ish bytes)
    let random_bytes: [u8; 16] = [
        0xde, 0xad, 0xbe, 0xef,
        0xca, 0xfe, 0xba, 0xbe,
        0xde, 0xad, 0xc0, 0xde,
        0xfe, 0xed, 0xfa, 0xce,
    ];
    let random_addr = sp - 16;
    unsafe {
        core::ptr::copy_nonoverlapping(random_bytes.as_ptr(), random_addr as *mut u8, 16);
    }
    sp = random_addr;

    // Step 3: align stack to 16 bytes
    sp &= !15;

    // Step 4: push aux vector entries (from last to first)
    unsafe {
        push_u32(&mut sp, AT_NULL); push_u32(&mut sp, 0);
        push_u32(&mut sp, AT_RANDOM); push_u32(&mut sp, random_addr);
        push_u32(&mut sp, AT_ENTRY); push_u32(&mut sp, hdr.e_entry);
        push_u32(&mut sp, AT_PAGESZ); push_u32(&mut sp, PAGE_SIZE as u32);
        push_u32(&mut sp, AT_PHNUM); push_u32(&mut sp, phnum as u32);
        push_u32(&mut sp, AT_PHENT); push_u32(&mut sp, core::mem::size_of::<Elf32Phdr>() as u32);
        push_u32(&mut sp, AT_PHDR); push_u32(&mut sp, phdr_addr);
        push_u32(&mut sp, AT_BASE); push_u32(&mut sp, 0);
    }

    // Step 5: push NULL (end of envp)
    unsafe { push_u32(&mut sp, 0); }

    // Step 6: push NULL (end of argv)
    unsafe { push_u32(&mut sp, 0); }

    // Step 7: push argv[0] pointer
    unsafe { push_u32(&mut sp, fname_addr); }

    // Step 8: push argc
    unsafe { push_u32(&mut sp, 1); }

    let entry = hdr.e_entry;
    let pd_phys = pd.phys_addr();
    crate::serial_driver::sprintln!(
        "  elf: loaded, entry={:#x}, sp={:#x}, pd={:#x}",
        entry, sp, pd_phys
    );

    // Restore kernel's page directory before returning
    unsafe { core::arch::asm!("mov cr3, {pd}", pd = in(reg) saved_pd); }

    Ok((pd_phys, entry, sp))
}

pub unsafe fn jump_to_user_entry(pd: u32, entry: u32, sp: u32) -> ! {
    crate::serial_driver::sprintln!("  elf: jumping to user mode entry={:#x} sp={:#x} pd={:#x}", entry, sp, pd);
    crate::arch::i686::paging::jump_to_user(pd, entry, sp)
}

// exec_elf: like load_elf but takes argv/envp from user space and sets up the stack with them
pub fn exec_elf(data: &[u8], argv: u32, envp: u32) -> Result<(u32, u32, u32), ElfError> {
    if data.len() < core::mem::size_of::<Elf32Header>() {
        return Err(ElfError::BadMagic);
    }

    let hdr: &Elf32Header = unsafe { &*(data.as_ptr() as *const Elf32Header) };

    if hdr.e_ident[0..4] != ELF_MAGIC {
        return Err(ElfError::BadMagic);
    }
    if hdr.e_ident[4] != 1 {
        return Err(ElfError::Not32Bit);
    }
    if hdr.e_type != 2 {
        return Err(ElfError::NotExec);
    }

    let phoff = hdr.e_phoff as usize;
    let phentsize = hdr.e_phentsize as usize;
    let phnum = hdr.e_phnum as usize;

    if phnum == 0 {
        return Err(ElfError::NoSegments);
    }

    let mut pd = unsafe { PageDir::new().ok_or(ElfError::PagingError)? };
    unsafe { pd.identity_map_kernel(); }

    // Collect argv strings from user space
    let mut argv_strings: alloc::vec::Vec<alloc::vec::Vec<u8>> = alloc::vec::Vec::new();
    if argv != 0 {
        let mut i = 0u32;
        loop {
            let ptr = unsafe { *((argv + i * 4) as *const u32) };
            if ptr == 0 { break; }
            let mut s: alloc::vec::Vec<u8> = alloc::vec::Vec::new();
            let mut p = ptr;
            loop {
                let c = unsafe { *(p as *const u8) };
                if c == 0 { break; }
                s.push(c);
                p += 1;
            }
            s.push(0);
            argv_strings.push(s);
            i += 1;
        }
    }

    // Collect envp strings from user space
    let mut envp_strings: alloc::vec::Vec<alloc::vec::Vec<u8>> = alloc::vec::Vec::new();
    if envp != 0 {
        let mut i = 0u32;
        loop {
            let ptr = unsafe { *((envp + i * 4) as *const u32) };
            if ptr == 0 { break; }
            let mut s: alloc::vec::Vec<u8> = alloc::vec::Vec::new();
            let mut p = ptr;
            loop {
                let c = unsafe { *(p as *const u8) };
                if c == 0 { break; }
                s.push(c);
                p += 1;
            }
            s.push(0);
            envp_strings.push(s);
            i += 1;
        }
    }

    let saved_pd: u32;
    unsafe { core::arch::asm!("mov {}, cr3", out(reg) saved_pd); }
    unsafe { core::arch::asm!("mov cr3, {pd}", pd = in(reg) pd.phys_addr()); }

    macro_rules! restore_and_return {
        ($e:expr) => {{
            unsafe { core::arch::asm!("mov cr3, {pd}", pd = in(reg) saved_pd); }
            return $e;
        }};
    }

    let mut phdr_addr = 0u32;

    for i in 0..phnum {
        let phdr_off = phoff + i * phentsize;
        if phdr_off + core::mem::size_of::<Elf32Phdr>() > data.len() {
            continue;
        }
        let phdr: &Elf32Phdr = unsafe { &*(data.as_ptr().add(phdr_off) as *const Elf32Phdr) };

        if phdr.p_type == PT_LOAD {
            let vaddr = phdr.p_vaddr as usize;
            let filesz = phdr.p_filesz as usize;
            let memsz = phdr.p_memsz as usize;
            let offset = phdr.p_offset as usize;

            let page_start = vaddr & !0xFFF;
            let page_end = (vaddr + memsz + 0xFFF) & !0xFFF;

            let mut page = page_start;
            while page < page_end {
                match unsafe { pd.map_alloc(page, true) } {
                    Some(_) => {}
                    None => {
                        restore_and_return!(Err(ElfError::LoadError));
                    }
                }
                page += 4096;
            }

            let avail = data.len().saturating_sub(offset);
            let copy_len = core::cmp::min(filesz, avail);
            if copy_len > 0 {
                unsafe {
                    core::ptr::copy_nonoverlapping(
                        data.as_ptr().add(offset),
                        vaddr as *mut u8,
                        copy_len,
                    );
                }
            }

            let bss_start = vaddr + filesz;
            let bss_end = vaddr + memsz;
            if bss_end > bss_start {
                unsafe {
                    core::ptr::write_bytes(bss_start as *mut u8, 0, bss_end - bss_start);
                }
            }

            if phdr.p_offset <= hdr.e_phoff && hdr.e_phoff < phdr.p_offset + phdr.p_filesz {
                phdr_addr = hdr.e_phoff - phdr.p_offset + phdr.p_vaddr;
            }
        }
    }

    // Allocate user stack
    if !unsafe { pd.alloc_stack(STACK_TOP as usize, STACK_SIZE) } {
        restore_and_return!(Err(ElfError::StackError));
    }

    let mut sp = STACK_TOP;

    // Push envp strings
    let mut envp_addrs: alloc::vec::Vec<u32> = alloc::vec::Vec::new();
    for s in envp_strings.iter().rev() {
        let len = s.len() as u32;
        sp -= len;
        unsafe {
            core::ptr::copy_nonoverlapping(s.as_ptr(), sp as *mut u8, s.len());
        }
        envp_addrs.push(sp);
    }
    envp_addrs.reverse();

    // Push argv strings
    let mut argv_addrs: alloc::vec::Vec<u32> = alloc::vec::Vec::new();
    for s in argv_strings.iter().rev() {
        let len = s.len() as u32;
        sp -= len;
        unsafe {
            core::ptr::copy_nonoverlapping(s.as_ptr(), sp as *mut u8, s.len());
        }
        argv_addrs.push(sp);
    }
    argv_addrs.reverse();

    // Align stack
    sp &= !15;

    // Push aux vector
    unsafe {
        push_u32(&mut sp, AT_NULL); push_u32(&mut sp, 0);
        push_u32(&mut sp, AT_ENTRY); push_u32(&mut sp, hdr.e_entry);
        push_u32(&mut sp, AT_PAGESZ); push_u32(&mut sp, PAGE_SIZE as u32);
        push_u32(&mut sp, AT_PHNUM); push_u32(&mut sp, phnum as u32);
        push_u32(&mut sp, AT_PHENT); push_u32(&mut sp, core::mem::size_of::<Elf32Phdr>() as u32);
        push_u32(&mut sp, AT_PHDR); push_u32(&mut sp, phdr_addr);
        push_u32(&mut sp, AT_BASE); push_u32(&mut sp, 0);
    }

    // Push envp pointers (NULL-terminated)
    unsafe { push_u32(&mut sp, 0); }
    for addr in envp_addrs.iter().rev() {
        unsafe { push_u32(&mut sp, *addr); }
    }

    // Push argv pointers (NULL-terminated)
    unsafe { push_u32(&mut sp, 0); }
    for addr in argv_addrs.iter().rev() {
        unsafe { push_u32(&mut sp, *addr); }
    }

    // Push argc
    unsafe { push_u32(&mut sp, argv_addrs.len() as u32); }

    let entry = hdr.e_entry;
    let pd_phys = pd.phys_addr();

    unsafe { core::arch::asm!("mov cr3, {pd}", pd = in(reg) saved_pd); }

    Ok((pd_phys, entry, sp))
}
