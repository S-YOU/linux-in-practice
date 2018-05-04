extern crate nix;

use std::io::{self, Write};
use std::os::raw::c_char;
use std::process::exit;
use std::ptr;

use nix::unistd::{ForkResult, Pid, fork, getpid};
use nix::libc::{self, EXIT_SUCCESS};

// nix::unistd::execve()'s argments are CString type.
fn execve(path: *const c_char, args: *const *const c_char, env: *const *const c_char) {
    unsafe {
        libc::execve(path, args, env);
    }
}

#[inline]
fn as_c_char(bytes: &[u8]) -> *const c_char {
    bytes.as_ptr() as *const c_char
}

fn child() {
    let args = [as_c_char(b"/bin/echo\0"), as_c_char(b"hello\0"), ptr::null()];
    println!("I'm child! my pid is {}.", getpid());
    let _ = io::stdout().flush();
    execve(as_c_char(b"/bin/echo\0"),
           &args as *const *const c_char,
           ptr::null());
    panic!("exec() failed");
}

fn parent(pid_c: Pid) {
    println!("I'm parent! my pid is {} and the pid of my child is {}.",
             getpid(),
             pid_c);
    exit(EXIT_SUCCESS);
}

fn main() {
    match fork().expect("fork() failed") {
        ForkResult::Parent { child } => {
            parent(child);
        }
        ForkResult::Child => {
            child();
        }
    }
    unreachable!();
}
