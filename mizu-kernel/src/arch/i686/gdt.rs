use core::mem;

extern "C" {
    fn gdt_reload(ptr: *const GdtPtr);
    fn tss_load(sel: u16);
}

#[repr(C, packed)]
struct GdtEntry(u64);

#[repr(C, packed)]
pub struct GdtPtr {
    pub limit: u16,
    pub base: u32,
}

#[repr(C, packed)]
pub struct Tss {
    prev: u16,
    _r1: u16,
    pub esp0: u32,
    pub ss0: u16,
    _r2: u16,
    esp1: u32,
    ss1: u16,
    _r3: u16,
    esp2: u32,
    ss2: u16,
    _r4: u16,
    cr3: u32,
    eip: u32,
    eflags: u32,
    eax: u32,
    ecx: u32,
    edx: u32,
    ebx: u32,
    esp: u32,
    ebp: u32,
    esi: u32,
    edi: u32,
    es: u16,
    _r5: u16,
    cs: u16,
    _r6: u16,
    ss: u16,
    _r7: u16,
    ds: u16,
    _r8: u16,
    fs: u16,
    _r9: u16,
    gs: u16,
    _r10: u16,
    ldt: u16,
    _r11: u16,
    _trap: u16,
    iobp: u16,
}

pub const KERNEL_CS: u16 = 0x08;
pub const KERNEL_DS: u16 = 0x10;
pub const USER_CS: u16 = 0x18 | 3;
pub const USER_DS: u16 = 0x20 | 3;
pub const TSS_SEL: u16 = 0x28;

const fn gdt_entry(base: u32, limit: u32, access: u8, flags: u8) -> u64 {
    let b1 = (base & 0xFFFF) as u64;
    let b2 = ((base >> 16) & 0xFF) as u64;
    let b3 = ((base >> 24) & 0xFF) as u64;
    let l1 = (limit & 0xFFFF) as u64;
    let l2 = ((limit >> 16) & 0x0F) as u64;
    l1 | (b1 << 16) | (b2 << 32) | ((access as u64) << 40) | ((l2 | ((flags as u64) << 4)) << 48) | (b3 << 56)
}

fn tss_entry(base: u32, limit: u32) -> u64 {
    let access: u8 = 0x89;
    let flags: u8 = 0x00;
    let b1 = (base & 0xFFFF) as u64;
    let b2 = ((base >> 16) & 0xFF) as u64;
    let b3 = ((base >> 24) & 0xFF) as u64;
    let l1 = (limit & 0xFFFF) as u64;
    let l2 = ((limit >> 16) & 0x0F) as u64;
    l1 | (b1 << 16) | (b2 << 32) | ((access as u64) << 40) | ((l2 | ((flags as u64) << 4)) << 48) | (b3 << 56)
}

pub static mut TSS: Tss = Tss {
    prev: 0, _r1: 0, esp0: 0, ss0: 0, _r2: 0,
    esp1: 0, ss1: 0, _r3: 0, esp2: 0, ss2: 0, _r4: 0,
    cr3: 0, eip: 0, eflags: 0,
    eax: 0, ecx: 0, edx: 0, ebx: 0, esp: 0, ebp: 0, esi: 0, edi: 0,
    es: 0, _r5: 0, cs: 0, _r6: 0, ss: 0, _r7: 0, ds: 0, _r8: 0,
    fs: 0, _r9: 0, gs: 0, _r10: 0, ldt: 0, _r11: 0, _trap: 0, iobp: 0,
};

static mut GDT: [GdtEntry; 6] = [
    GdtEntry(0),
    GdtEntry(gdt_entry(0, 0xFFFFF, 0x9A, 0xC)), // kernel code
    GdtEntry(gdt_entry(0, 0xFFFFF, 0x92, 0xC)), // kernel data
    GdtEntry(gdt_entry(0, 0xFFFFF, 0xFA, 0xC)), // user code
    GdtEntry(gdt_entry(0, 0xFFFFF, 0xF2, 0xC)), // user data
    GdtEntry(0), // TSS placeholder
];

pub fn init(kstack: u32) {
    unsafe {
        TSS.ss0 = KERNEL_DS;
        TSS.esp0 = kstack;

        let tss_addr = &TSS as *const _ as u32;
        let tss_size = mem::size_of::<Tss>() as u32;
        GDT[5] = GdtEntry(tss_entry(tss_addr, tss_size - 1));

        let ptr = GdtPtr { limit: (mem::size_of::<[GdtEntry; 6]>() - 1) as u16, base: &GDT as *const _ as u32 };
        gdt_reload(&ptr as *const GdtPtr);
        tss_load(TSS_SEL);
    }
}
