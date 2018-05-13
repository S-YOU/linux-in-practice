extern crate nix;

use std::ffi::CStr;
use std::io::{Write, stdout};
use std::os::unix::io::RawFd;
use std::process::Command;
use std::ptr;

use nix::libc::{c_char, c_void, memcpy};
use nix::fcntl::{OFlag, open};
use nix::sys::mman::{MapFlags, ProtFlags, mmap, munmap};
use nix::sys::stat::Mode;
use nix::unistd::{close, getpid};

const ALLOC_SIZE: usize = 100 * 1024 * 1024;

struct Fd(RawFd);
impl Fd {
    fn new(fd: RawFd) -> Fd {
        Fd(fd)
    }
}
impl Drop for Fd {
    fn drop(&mut self) {
        close(self.0).expect("close() failed");
        self.0 = -1;
    }
}

fn main() {
    let pid = getpid();
    let arg = format!("/proc/{}/maps", pid);

    println!("*** memory map before mapping file ***");
    let _ = stdout().flush();
    Command::new("cat").arg(&arg).status().expect("failed to execute process");

    let fd = Fd::new(open("testfile", OFlag::O_RDWR, Mode::empty()).expect("open() failed"));
    let file_contents = unsafe {
        mmap(ptr::null_mut(),
             ALLOC_SIZE,
             ProtFlags::PROT_READ | ProtFlags::PROT_WRITE,
             MapFlags::MAP_SHARED,
             fd.0,
             0)
            .expect("mmap() failed")
    };

    println!("");
    println!("*** succeeded to map file: address = {:?}; size = {:#x} ***",
             file_contents,
             ALLOC_SIZE);

    println!("");
    println!("*** memory map after mapping file ***");
    let _ = stdout().flush();
    Command::new("cat").arg(&arg).status().expect("failed to execute process");

    println!("");
    println!("*** file contents before overwrite mapped region: {}",
             unsafe { CStr::from_ptr(file_contents as *const c_char) }
                 .to_str()
                 .expect("UTF-8 error."));

    // overwrite mapped region
    let overwrite_data = "HELLO";
    unsafe {
        memcpy(file_contents,
               overwrite_data.as_ptr() as *const c_void,
               overwrite_data.len());
    }

    println!("*** overwritten mapped region with: {}",
             unsafe { CStr::from_ptr(file_contents as *const c_char) }
                 .to_str()
                 .expect("UTF-8 error."));

    unsafe {
        munmap(file_contents, ALLOC_SIZE).expect("munmap() failed");
    }
}
