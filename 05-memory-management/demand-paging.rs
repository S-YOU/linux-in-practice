extern crate nix;
extern crate time;

use std::io::{Read, stdin};

use nix::unistd::sleep;

const BUFFER_SIZE: usize = 100 * 1024 * 1024;
const NCYCLE: usize = 10;
const PAGE_SIZE: usize = 4096;

fn main() {
    let t = time::now();
    let mut getchar = stdin().bytes();
    println!("{}: before allocation, please press Enter key", t.ctime());
    getchar.next();

    let mut p = Vec::with_capacity(BUFFER_SIZE);
    unsafe {
        p.set_len(BUFFER_SIZE);
    }

    let t = time::now();
    println!("{}: allocated {}MB, please press Enter key",
             t.ctime(),
             BUFFER_SIZE / (1024 * 1024));
    getchar.next();

    let mut i = 0;
    while i < BUFFER_SIZE {
        p[i] = 0u8;
        let cycle = i / (BUFFER_SIZE / NCYCLE);
        if cycle != 0 && i % (BUFFER_SIZE / NCYCLE) == 0 {
            let t = time::now();
            println!("{}: touched {}MB", t.ctime(), i / (1024 * 1024));
            sleep(1);
        }
        i += PAGE_SIZE;
    }
    let t = time::now();
    println!("{}: touched {}MB, please press Enter key",
             t.ctime(),
             BUFFER_SIZE / (1024 * 1024));
    getchar.next();
}
