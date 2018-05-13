extern crate nix;

use std::io::{Write, stdout};
use std::process::{Command, Stdio, exit};
use std::slice::from_raw_parts_mut;

use nix::libc::EXIT_SUCCESS;
use nix::sys::wait::wait;
use nix::unistd::{ForkResult, fork, getpid};

const BUFFER_SIZE: usize = 100 * 1024 * 1024;
const PAGE_SIZE: usize = 4096;

fn child_fn(p: *mut u8) {
    let p = unsafe { from_raw_parts_mut(p, BUFFER_SIZE) };
    let greparg = format!("^{}", getpid());

    println!("*** child ps info before memory access ***:");
    let _ = stdout().flush();
    Command::new("grep")
        .arg(&greparg)
        .stdin(Command::new("ps")
            .args(&["-o", "pid,comm,vsz,rss,min_flt,maj_flt"])
            .stdout(Stdio::piped())
            .spawn()
            .expect("Failed to start ps")
            .stdout
            .unwrap())
        .status()
        .expect("Failed to start grep");

    println!("*** free memory info before memory access ***:");
    let _ = stdout().flush();
    Command::new("free").status().expect("Failed to start free");

    let mut i = 0;
    while i < BUFFER_SIZE {
        p[i] = 0u8;
        i += PAGE_SIZE;
    }
    println!("*** child ps info after memory access ***:");
    let _ = stdout().flush();
    Command::new("grep")
        .arg(&greparg)
        .stdin(Command::new("ps")
            .args(&["-o", "pid,comm,vsz,rss,min_flt,maj_flt"])
            .stdout(Stdio::piped())
            .spawn()
            .expect("Failed to start ps")
            .stdout
            .unwrap())
        .status()
        .expect("Failed to start grep");

    println!("*** free memory info after memory access ***:");
    let _ = stdout().flush();
    Command::new("free").status().expect("Failed to start free");
}

fn parent_fn() {
    let _ = wait();
    exit(EXIT_SUCCESS);
}

fn main() {
    let mut p = Vec::with_capacity(BUFFER_SIZE);
    unsafe {
        p.set_len(BUFFER_SIZE);
    }

    let mut i = 0;
    while i < BUFFER_SIZE {
        p[i] = 0u8;
        i += PAGE_SIZE;
    }
    println!("*** free memory info before fork ***:");
    let _ = stdout().flush();
    let _ = Command::new("free").status();

    match fork().expect("fork() failure") {
        ForkResult::Parent { child: _ } => parent_fn(),
        ForkResult::Child => child_fn(p.as_mut_ptr() as *mut u8),
    }
}
