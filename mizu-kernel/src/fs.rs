use alloc::string::String;
use alloc::vec::Vec;
use core::str;

use crate::serial_driver;
use crate::vga_driver;

#[derive(Debug, Clone)]
pub struct FileInfo {
    pub name: String,
    pub size: usize,
    pub data: *const u8,
}

static mut FILES: core::option::Option<Vec<FileInfo>> = None;

struct RamFile {
    name: String,
    data: Vec<u8>,
}

static mut RAM_FILES: Option<Vec<RamFile>> = None;

extern "C" {
    static _binary_initramfs_tar_start: u8;
    static _binary_initramfs_tar_end: u8;
}

/// Init filesystem from embedded initramfs
pub fn init() {
    let start = unsafe { &_binary_initramfs_tar_start as *const u8 as usize };
    let end = unsafe { &_binary_initramfs_tar_end as *const u8 as usize };
    let size = end - start;

    serial_driver::sprintln!("  fs: initramfs at {:#x}-{:#x} ({} bytes)", start, end, size);

    let mut files = Vec::new();
    parse_tar(start as *const u8, size, &mut files);

    unsafe { FILES = Some(files); }

    serial_driver::sprintln!("  fs: {} files loaded", file_count());
    vga_driver::write_str("  fs: OK\n");
}

fn parse_tar(data: *const u8, size: usize, files: &mut Vec<FileInfo>) {
    let mut offset: usize = 0;

    while offset + 512 <= size {
        let header = unsafe { core::slice::from_raw_parts(data.add(offset), 512) };

        if header[0] == 0 {
            break;
        }

        let name = trim_str(&header[0..100]);
        if name.is_empty() || name == "." || name == ".." {
            offset += 512;
            continue;
        }

        let size_str = trim_str(&header[124..136]);
        let file_size = parse_octal(&size_str);

        let data_offset = offset + 512;
        let padded_size = (file_size + 511) & !511;

        if file_size > 0 && data_offset + padded_size <= size {
            let file_data = unsafe { data.add(data_offset) };
            files.push(FileInfo {
                name: String::from(name),
                size: file_size,
                data: file_data,
            });
        }

        offset = data_offset + padded_size;
    }
}

fn trim_str(s: &[u8]) -> &str {
    let end = s.iter().position(|&b| b == 0).unwrap_or(s.len());
    str::from_utf8(&s[..end]).unwrap_or("")
}

fn parse_octal(s: &str) -> usize {
    s.chars()
        .filter(|c| *c >= '0' && *c <= '7')
        .fold(0, |n, c| n * 8 + (c as usize - '0' as usize))
}

fn file_count() -> usize {
    unsafe { FILES.as_ref().map(|v| v.len()).unwrap_or(0) }
}

pub fn file_list() -> Vec<String> {
    let mut out = Vec::new();
    if let Some(ref files) = unsafe { &FILES } {
        for f in files {
            out.push(f.name.clone());
        }
    }
    if let Some(ref ramfs) = unsafe { &RAM_FILES } {
        for f in ramfs {
            if !out.contains(&f.name) {
                out.push(f.name.clone());
            }
        }
    }
    out
}

pub fn file_read(name: &str) -> Option<&'static [u8]> {
    if let Some(ref ramfs) = unsafe { &RAM_FILES } {
        for f in ramfs {
            if f.name == name {
                return Some(unsafe { core::slice::from_raw_parts(f.data.as_ptr(), f.data.len()) });
            }
        }
    }
    if let Some(ref files) = unsafe { &FILES } {
        for f in files {
            if f.name == name {
                return Some(unsafe { core::slice::from_raw_parts(f.data, f.size) });
            }
        }
    }
    None
}

pub fn file_exists(name: &str) -> bool {
    if let Some(ref ramfs) = unsafe { &RAM_FILES } {
        for f in ramfs {
            if f.name == name { return true; }
        }
    }
    if let Some(ref files) = unsafe { &FILES } {
        for f in files {
            if f.name == name { return true; }
        }
    }
    false
}

fn ramfs_init() {
    unsafe { RAM_FILES = Some(Vec::new()); }
}

pub fn file_write(name: &str, data: &[u8]) -> bool {
    unsafe {
        if RAM_FILES.is_none() { ramfs_init(); }
        let ramfs = RAM_FILES.as_mut().unwrap();
        ramfs.retain(|f| f.name != name);
        ramfs.push(RamFile {
            name: String::from(name),
            data: Vec::from(data),
        });
    }
    true
}

pub fn file_remove(name: &str) -> bool {
    unsafe {
        if let Some(ref mut ramfs) = RAM_FILES {
            let len_before = ramfs.len();
            ramfs.retain(|f| f.name != name);
            ramfs.len() < len_before
        } else {
            false
        }
    }
}

pub fn file_rename(old: &str, new: &str) -> bool {
    unsafe {
        if let Some(ref mut ramfs) = RAM_FILES {
            if let Some(f) = ramfs.iter_mut().find(|f| f.name == old) {
                f.name = String::from(new);
                return true;
            }
        }
        false
    }
}

pub fn sys_getdents(fd: u32, dent: u32, count: u32) -> i32 {
    if fd != 0 { return -crate::syscall::EBADF; }
    let files = match unsafe { &FILES } {
        Some(f) => f,
        None => return 0,
    };
    let mut written = 0usize;
    let base = dent as *mut u8;
    let mut ino: u64 = 1;
    for f in files {
        let name_bytes = f.name.as_bytes();
        let reclen = 24 + name_bytes.len() + 1;
        if written + reclen > count as usize { break; }
        unsafe {
            let p = base.add(written);
            *(p as *mut u64) = ino;           // d_ino
            *(p.add(8) as *mut u64) = 0;       // d_off
            *(p.add(16) as *mut u16) = reclen as u16; // d_reclen
            *(p.add(18) as *mut u8) = 0;       // d_type (DT_UNKNOWN)
            for (i, &b) in name_bytes.iter().enumerate() {
                p.add(19 + i).write(b);
            }
            p.add(19 + name_bytes.len()).write(0); // null terminator
        }
        written += reclen;
        ino += 1;
    }
    written as i32
}
