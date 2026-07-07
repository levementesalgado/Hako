use core::arch::asm;

const PAGE_SIZE: usize = 4096;
const PTES: usize = 1024;
const PDES: usize = 1024;

const PDE_P: u32 = 1 << 0;
const PDE_RW: u32 = 1 << 1;
const PDE_US: u32 = 1 << 2;

const PTE_P: u32 = 1 << 0;
const PTE_RW: u32 = 1 << 1;
const PTE_US: u32 = 1 << 2;

pub struct PageDir {
    phys: *mut u32,
}

impl PageDir {
    pub unsafe fn new() -> Option<Self> {
        let frame = crate::memory::alloc_frame()?;
        let pd = frame as *mut u32;
        for i in 0..PDES {
            pd.add(i).write(0);
        }
        Some(PageDir { phys: pd })
    }

    pub unsafe fn identity_map_kernel(&mut self) -> bool {
        // Kernel loaded at 1MB, map 3 PDEs (12MB) to cover kernel + space for stacks.
        // This gives us 0-12MB identity-mapped, enough for PID 1 and 2 kernel stacks.
        let pdes_needed = 3;
        for pde_idx in 0..pdes_needed {
            let pt_frame = match crate::memory::alloc_frame() {
                Some(f) => f,
                None => return false,
            };
            let pt = pt_frame as *mut u32;
            for i in 0..PTES {
                let vaddr = (pde_idx * PTES + i) * PAGE_SIZE;
                let pte = vaddr as u32 | PTE_P | PTE_RW;
                pt.add(i).write(pte);
            }
            self.phys.add(pde_idx).write(pt_frame as u32 | PDE_P | PDE_RW | PDE_US);
        }
        true
    }

    pub unsafe fn map_alloc(&mut self, vaddr: usize, user: bool) -> Option<usize> {
        let frame = crate::memory::alloc_frame()?;
        if self.map(vaddr, frame, user) {
            Some(frame)
        } else {
            None
        }
    }

    pub unsafe fn map(&mut self, vaddr: usize, paddr: usize, user: bool) -> bool {
        let pdi = (vaddr / PAGE_SIZE) / PTES;
        let pti = (vaddr / PAGE_SIZE) % PTES;

        let pde = self.phys.add(pdi).read();
        let pt_frame = if pde & PDE_P == 0 {
            let frame = match crate::memory::alloc_frame() {
                Some(f) => f,
                None => return false,
            };
            let pt = frame as *mut u32;
            for i in 0..PTES {
                pt.add(i).write(0);
            }
            let flags = PDE_P | PDE_RW | if user { PDE_US } else { 0 };
            self.phys.add(pdi).write(frame as u32 | flags);
            frame
        } else {
            (pde & 0xFFFFF000) as usize
        };

        let pt = pt_frame as *mut u32;
        let flags = PTE_P | PTE_RW | if user { PTE_US } else { 0 };
        pt.add(pti).write(paddr as u32 | flags);
        true
    }

    pub unsafe fn alloc_stack(&mut self, top: usize, size: usize) -> bool {
        let start = top - size;
        let pages = size / PAGE_SIZE;
        for i in 0..pages {
            let frame = match crate::memory::alloc_frame() {
                Some(f) => f,
                None => return false,
            };
            let vaddr = start + i * PAGE_SIZE;
            if !self.map(vaddr, frame, true) {
                return false;
            }
        }
        true
    }

    pub fn phys_addr(&self) -> u32 {
        self.phys as u32
    }

    pub unsafe fn from_phys(phys: u32) -> Self {
        PageDir { phys: phys as *mut u32 }
    }
}

pub unsafe fn jump_to_user(pd: u32, entry: u32, stack_top: u32) -> ! {
    let tmp: u32;
    asm!("mov cr3, {pd}", "mov {tmp}, cr0", pd = in(reg) pd, tmp = out(reg) tmp);
    asm!("or {tmp}, 0x80000000", "mov cr0, {tmp}", tmp = in(reg) tmp | 0x80000000, options(nostack));
    asm!(
        "mov eax, 0x23",
        "push eax",
        "push {sp}",
        "pushfd",
        "pop eax",
        "or eax, 0x200",
        "push eax",
        "mov eax, 0x1B",
        "push eax",
        "push {entry}",
        "iretd",
        sp = in(reg) stack_top,
        entry = in(reg) entry,
        in("eax") 0,
        options(noreturn)
    );
}
