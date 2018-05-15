extern crate nix;

use std::{env, ptr};
use std::mem::uninitialized;
use std::process::exit;
use std::slice::from_raw_parts_mut;

use nix::libc::{self, CLOCK_MONOTONIC, EXIT_FAILURE, timespec};
use nix::sys::mman::{MapFlags, ProtFlags, mmap, munmap};

const CACHE_LINE_SIZE: usize = 64;
const NLOOP: usize = 4 * 1024 * 1024 * 1024;
const NSECS_PER_SEC: i64 = 1000000000;

#[inline]
fn diff_nsec(before: &timespec, after: &timespec) -> i64 {
    (after.tv_sec * NSECS_PER_SEC + after.tv_nsec) -
    (before.tv_sec * NSECS_PER_SEC + before.tv_nsec)
}

fn get_monotonic_time() -> Result<timespec, i32> {
    let mut tp: timespec = unsafe { uninitialized() };
    let ret = unsafe { libc::clock_gettime(CLOCK_MONOTONIC, &mut tp as *mut timespec) };
    if ret == 0 {
        Ok(tp)
    } else {
        Err(nix::errno::errno())
    }
}

fn main() {
    let size: usize;
    match env::args().nth(1).and_then(|s| s.parse::<usize>().ok()) {
        Some(n) => {
            size = n * 1024;
        }
        None => {
            let progname = env::args().nth(0).unwrap();
            eprintln!("usage: {} <size[kb]>", progname);
            exit(EXIT_FAILURE);
        }
    }
    if size == 0 {
        eprintln!("size should be >= 1: {}", size);
        exit(EXIT_FAILURE);
    }
    let buffer = unsafe {
            mmap(ptr::null_mut(),
                 size,
                 ProtFlags::PROT_READ | ProtFlags::PROT_WRITE,
                 MapFlags::MAP_PRIVATE | MapFlags::MAP_ANONYMOUS,
                 -1,
                 0)
        }
        .expect("mmap() failed");

    {
        let buffer = unsafe { from_raw_parts_mut(buffer as *mut u8, size) };
        let before = get_monotonic_time().unwrap();
        for _i in 0..NLOOP / (size / CACHE_LINE_SIZE) {
            let mut j = 0;
            while j < size {
                unsafe {
                    *(buffer.get_unchecked_mut(j)) = 0;
                }
                j += CACHE_LINE_SIZE;
            }
        }
        let after = get_monotonic_time().unwrap();
        println!("{}", diff_nsec(&before, &after) as f64 / NLOOP as f64);
    }

    unsafe { munmap(buffer, size) }.expect("munmap() failed");
}
