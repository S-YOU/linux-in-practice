extern crate nix;

use std::process::Command;
use std::ptr;

use nix::unistd::getpid;
use nix::sys::mman::{MapFlags, ProtFlags, mmap, munmap};

const ALLOC_SIZE: usize = 100 * 1024 * 1024;

fn main() {
    let pid = getpid();
    println!("*** memory map before memory allocation ***");
    Command::new("cat")
        .arg(format!("/proc/{}/maps", pid))
        .status()
        .expect("failed to execute process");

    let new_memory = unsafe {
        mmap(ptr::null_mut(),
             ALLOC_SIZE,
             ProtFlags::PROT_READ | ProtFlags::PROT_WRITE,
             MapFlags::MAP_PRIVATE | MapFlags::MAP_ANONYMOUS,
             -1,
             0)
            .expect("mmap() failed")
    };
    println!("");
    println!("*** succeeded to allocate memory: address = 0x{:x}; size = 0x{:x} ***",
             new_memory as usize,
             ALLOC_SIZE);

    println!("");
    println!("*** memory map after memory allocation ***");
    Command::new("cat")
        .arg(format!("/proc/{}/maps", pid))
        .status()
        .expect("failed to execute process");

    unsafe {
        munmap(new_memory, ALLOC_SIZE).expect("munmap() failed");
    }
}
