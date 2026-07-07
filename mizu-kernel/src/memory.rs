use core::alloc::{GlobalAlloc, Layout};
use core::ptr;

use crate::serial_driver;

const FRAME_SIZE: usize = 4096;
const HEAP_SIZE: usize = 64 * 1024;
const BITMAP_FRAMES: usize = 128 * 1024;

/* ─── Frame bitmap allocator ────────────────── */
static mut FRAME_BITMAP: [u8; BITMAP_FRAMES / 8] = [0; BITMAP_FRAMES / 8];
static mut TOTAL_FRAMES: usize = 0;
static mut USED_FRAMES: usize = 0;
pub static mut TOTAL_MEM: usize = 0;

pub fn total_frames() -> usize { unsafe { TOTAL_FRAMES } }
fn total() -> usize { unsafe { TOTAL_FRAMES } }
fn used() -> usize { unsafe { USED_FRAMES } }

fn set(i: usize, v: bool) {
    unsafe {
        if v { FRAME_BITMAP[i / 8] |= 1 << (i % 8); }
        else { FRAME_BITMAP[i / 8] &= !(1 << (i % 8)); }
    }
}

fn is_used(i: usize) -> bool {
    unsafe { FRAME_BITMAP[i / 8] & (1 << (i % 8)) != 0 }
}

/* ─── Inicialização ─────────────────────────── */
extern "C" {
    static _bss_end: u8;
}

fn kernel_end() -> usize {
    unsafe { (&_bss_end as *const u8 as usize + 0xFFF) & !0xFFF }
}

pub fn kernel_phys_end() -> usize {
    kernel_end()
}

pub fn init(mem_upper: u32) {
    let total_bytes = ((mem_upper as usize) + 1024) * 1024;
    unsafe {
        TOTAL_FRAMES = total_bytes / FRAME_SIZE;
        TOTAL_MEM = total_bytes;
    }
    let n = total();
    let kend = kernel_end();

    // Tudo usado por padrão
    for i in 0..n {
        set(i, true);
    }

    // Libera frames após o fim do kernel até o fim da memória
    let kframe = kend / FRAME_SIZE;
    for i in kframe..n {
        set(i, false);
    }

    // Frames 0-255: conventional memory (reserved by BIOS/VGA)

    unsafe { USED_FRAMES = kframe; }

    serial_driver::sprintln!("  memory: {} KB, {} free frames",
        total_bytes / 1024, n - kframe);

    init_heap();
}

/* ─── API ────────────────────────────────────── */
pub fn alloc_frame() -> Option<usize> {
    let n = total();
    for i in 0..n {
        if !is_used(i) {
            set(i, true);
            unsafe { USED_FRAMES += 1; }
            return Some(i * FRAME_SIZE);
        }
    }
    None
}

pub fn kernel_end_frame() -> usize {
    extern "C" { static _bss_end: u8; }
    let kend = unsafe { (&_bss_end as *const u8 as usize + 0xFFF) & !0xFFF };
    kend / FRAME_SIZE
}

pub fn alloc_frame_at(addr: usize) -> bool {
    let i = addr / FRAME_SIZE;
    if i >= total() { return false; }
    if is_used(i) { return false; }
    set(i, true);
    unsafe { USED_FRAMES += 1; }
    true
}

pub fn free_frame(addr: usize) {
    let i = addr / FRAME_SIZE;
    if i < total() {
        set(i, false);
        unsafe { USED_FRAMES -= 1; }
    }
}

pub fn stats() -> (usize, usize, usize) {
    (total(), used(), total() - used())
}

/* ─── Heap: free list com raw pointers ───────── */
struct Node {
    size: usize,
    next: *mut Node,
}

const NODE_SZ: usize = core::mem::size_of::<Node>();

static mut HEAP_BASE: usize = 0;
static mut HEAP_END: usize = 0;
static mut FREE: *mut Node = ptr::null_mut();

fn init_heap() {
    let base = kernel_end();
    let aligned = (base + 0xFFFFF) & !0xFFFFF;

    unsafe {
        HEAP_BASE = aligned;
        HEAP_END = aligned + HEAP_SIZE;

        let head = aligned as *mut Node;
        (*head).size = HEAP_SIZE - NODE_SZ;
        (*head).next = ptr::null_mut();
        FREE = head;

        // Marca frames do heap como usados no frame allocator
        let sf = aligned / FRAME_SIZE;
        let ef = (aligned + HEAP_SIZE + FRAME_SIZE - 1) / FRAME_SIZE;
        for i in sf..ef {
            set(i, true);
        }
        unsafe { USED_FRAMES += ef - sf; }
    }
}

unsafe fn alloc_from_heap(size: usize, _align: usize) -> *mut u8 {
    let size = (size + 7) & !7;
    let needed = size + NODE_SZ;

    let mut prev: *mut Node = ptr::null_mut();
    let mut curr = FREE;

    while curr != ptr::null_mut() {
        if (*curr).size >= needed {
            let ptr = (curr as usize + NODE_SZ) as *mut u8;
            let remaining = (*curr).size - needed;

            if remaining > NODE_SZ + 8 {
                // Split
                let next_node = (curr as usize + NODE_SZ + size) as *mut Node;
                (*next_node).size = remaining - NODE_SZ;
                (*next_node).next = (*curr).next;
                (*curr).size = size;
                (*curr).next = next_node;

                if prev != ptr::null_mut() {
                    (*prev).next = next_node;
                } else {
                    FREE = next_node;
                }
            } else {
                // Use whole block
                if prev != ptr::null_mut() {
                    (*prev).next = (*curr).next;
                } else {
                    FREE = (*curr).next;
                }
            }
            return ptr;
        }
        prev = curr;
        curr = (*curr).next;
    }

    // OOM: tenta recuperar FREE
    ptr::null_mut()
}

unsafe fn dealloc_from_heap(ptr: *mut u8) {
    if ptr.is_null() { return; }
    let node = (ptr as usize - NODE_SZ) as *mut Node;
    (*node).next = FREE;
    FREE = node;
}

/* ─── GlobalAlloc ────────────────────────────── */
pub struct MizuAlloc;

unsafe impl GlobalAlloc for MizuAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        alloc_from_heap(layout.size(), layout.align())
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        dealloc_from_heap(ptr);
    }
}

#[global_allocator]
pub static ALLOCATOR: MizuAlloc = MizuAlloc;
