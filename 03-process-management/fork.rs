extern crate nix;

use std::process::exit;

use nix::unistd::{ForkResult, Pid, fork, getpid};
use nix::libc::EXIT_SUCCESS;

fn child() {
    println!("I'm child! my pid is {}.", getpid());
    exit(EXIT_SUCCESS);
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
