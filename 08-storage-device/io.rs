#[macro_use]
extern crate nix;

extern crate rand;

use std::env;
use std::process::exit;
use std::ptr;
use std::slice::{from_raw_parts, from_raw_parts_mut};

use nix::libc::{EXIT_FAILURE, c_int, c_void, free, ioctl, posix_memalign};
use nix::fcntl::{OFlag, open};
use nix::sys::stat::Mode;
use nix::unistd::{Whence, close, fdatasync, lseek, read, write};
use rand::{weak_rng, Rng};

const PART_SIZE: usize = 1024 * 1024 * 1024;
const ACCESS_SIZE: usize = 64 * 1024 * 1024;

// linux/fs.h
const BLKSSZGET: u64 = io!(0x12, 104);
fn get_sector_size(fd: c_int) -> nix::Result<usize> {
    let mut size: usize = 0;
    nix::errno::Errno::result(unsafe { ioctl(fd, BLKSSZGET, &mut size) }).map(|_| size)
}

struct AlignedMem<T>(*mut T, usize);
impl<T> AlignedMem<T> {
    fn allocate(alignment: usize, size: usize) -> Result<AlignedMem<T>, i32> {
        let mut p: *mut c_void = ptr::null_mut();
        let e = unsafe { posix_memalign((&mut p) as *mut *mut c_void, alignment, size) };
        match e {
            0 => Ok(AlignedMem(p as *mut T, size)),
            _ => Err(e),
        }
    }
    fn as_slice(&self) -> &[T] {
        unsafe { from_raw_parts(self.0, self.1) }
    }
    fn as_slice_mut(&self) -> &mut [T] {
        unsafe { from_raw_parts_mut(self.0, self.1) }
    }
}
impl<T> Drop for AlignedMem<T> {
    fn drop(&mut self) {
        unsafe {
            free(self.0 as *mut c_void);
        }
    }
}

fn main() {
    let argv: Vec<_> = env::args().collect();

    if argv.len() != 6 {
        eprintln!("usage: {} <filename> <kernel's help> <r/w> <access pattern> <block size[KB]>",
                  argv[0]);
        exit(EXIT_FAILURE);
    }
    let filename = &argv[1];
    let help = match &*argv[2] {
        "on" => true,
        "off" => false,
        _ => {
            eprintln!("kernel's help should be 'on' or 'off': {}", argv[2]);
            exit(EXIT_FAILURE);
        }
    };

    let write_flag = match &*argv[3] {
        "w" => true,
        "r" => false,
        _ => {
            eprintln!("r/w should be 'r' or 'w': {}", argv[3]);
            exit(EXIT_FAILURE);
        }
    };

    let random = match &*argv[4] {
        "seq" => false,
        "rand" => true,
        _ => {
            eprintln!("access pattern should be 'seq' or 'rand': {}", argv[4]);
            exit(EXIT_FAILURE);
        }
    };

    let part_size = PART_SIZE;
    let access_size = ACCESS_SIZE;

    let block_size = argv[5].parse::<usize>().expect("parse error") * 1024;
    if block_size == 0 {
        eprintln!("block size should be > 0: {}", block_size);
        exit(EXIT_FAILURE);
    } else if access_size % block_size != 0 {
        eprintln!("access size({}) should be multiple of block size: {}",
                  access_size,
                  block_size);
        exit(EXIT_FAILURE);
    }

    let maxcount = part_size / block_size;
    let count = access_size / block_size;

    let mut offset = vec![0; maxcount];
    for (i, x) in offset.iter_mut().enumerate() {
        *x = i;
    }
    if random {
        let mut rng = weak_rng();
        rng.shuffle(&mut offset);
    }

    let mut flag = OFlag::O_RDWR | OFlag::O_EXCL;
    if !help {
        flag |= OFlag::O_DIRECT;
    }

    let fd = open(filename.as_str(), flag, Mode::empty()).expect("open() failed");

    let sector_size = get_sector_size(fd).expect("ioctl() failed");

    let buf = AlignedMem::allocate(sector_size, block_size).expect("posix_memalign() failed");
    {
        let _: &[u8] = buf.as_slice();
    }
    for i in 0..count {
        lseek(fd, (offset[i] * block_size) as i64, Whence::SeekSet).expect("lseek() failed");
        if write_flag {
            write(fd, buf.as_slice()).expect("write() failed");
        } else {
            read(fd, buf.as_slice_mut()).expect("read() failed");
        }
    }
    fdatasync(fd).expect("fdatasync() failed");
    close(fd).expect("close() failed");
}
