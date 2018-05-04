extern crate nix;

use std::env;
use std::mem::uninitialized;
use std::process::exit;

use nix::libc::{self, CLOCK_MONOTONIC, EXIT_FAILURE, EXIT_SUCCESS, timespec};
use nix::unistd::{ForkResult, fork};
use nix::sys::signal::{Signal, kill};
use nix::sys::wait::wait;

const NLOOP_FOR_ESTIMATION: u64 = 50_000_000;
const NSECS_PER_MSEC: u64 = 1_000_000;
const NSECS_PER_SEC: u64 = 1_000_000_000;

#[inline]
fn diff_nsec(before: &timespec, after: &timespec) -> i64 {
    (after.tv_sec * NSECS_PER_SEC as i64 + after.tv_nsec) -
    (before.tv_sec * NSECS_PER_SEC as i64 + before.tv_nsec)
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

fn loops_per_msec() -> u64 {
    let before = get_monotonic_time().unwrap();
    for _i in 0..NLOOP_FOR_ESTIMATION {}
    let after = get_monotonic_time().unwrap();
    NLOOP_FOR_ESTIMATION * NSECS_PER_MSEC / diff_nsec(&before, &after) as u64
}

#[inline]
fn load(nloop: u64) {
    for _i in 0..nloop {}
}

fn child_fn(id: u32,
            buf: &mut Vec<timespec>,
            nrecord: u32,
            nloop_per_resol: u64,
            start: &timespec) {
    for _i in 0..nrecord {
        load(nloop_per_resol);
        let ts = get_monotonic_time().unwrap();
        buf.push(ts);
    }
    for i in 0..nrecord {
        println!("{}\t{}\t{}",
                 id,
                 diff_nsec(start, &buf[i as usize]) / NSECS_PER_MSEC as i64,
                 (i + 1) * 100 / nrecord);
    }
    exit(EXIT_SUCCESS);
}

#[allow(dead_code)]
fn parent_fn(nproc: u32) {
    for _i in 0..nproc {
        let _ = wait();
    }
}


fn main() {
    let usage_then_exit = || {
        eprintln!("usage: {} <nproc> <total[ms]> <resolution[ms]>",
                  env::args().nth(0).unwrap());
        exit(EXIT_FAILURE);
    };
    let args: Vec<u32> = env::args()
        .skip(1)
        .map(|s| s.parse())
        .filter_map(|r| r.ok())
        .collect();

    if args.len() < 3 {
        usage_then_exit();
    }

    let nproc = args[0];
    let total = args[1];
    let resol = args[2];

    if nproc < 1 {
        eprintln!("<nproc>({}) should be >= 1", nproc);
        exit(EXIT_FAILURE);
    }
    if total < 1 {
        eprintln!("<total>({}) should be >= 1", total);
        exit(EXIT_FAILURE);
    }
    if resol < 1 {
        eprintln!("<resol>({}) should be >= 1", resol);
        exit(EXIT_FAILURE);
    }
    if total % resol != 0 {
        eprintln!("<total>({} should be multiple of <resolution>({})",
                  total,
                  resol);
        exit(EXIT_FAILURE);
    }
    let nrecord = total / resol;

    let mut logbuf = Vec::with_capacity(nrecord as usize);
    println!("estimating workload which takes just one milisecond");
    let nloop_per_resol = loops_per_msec() * resol as u64;
    println!("end estimation");

    let mut pids = Vec::with_capacity(nproc as usize);

    let mut exitcode = EXIT_SUCCESS;
    let start = get_monotonic_time().unwrap();
    let mut ncreated = 0;
    for i in 0..nproc {
        match fork() {
            Ok(ForkResult::Parent { child }) => {
                pids.push(child);
                ncreated += 1;
            }
            Ok(ForkResult::Child) => {
                child_fn(i, &mut logbuf, nrecord, nloop_per_resol, &start);
                unreachable!();
            }
            Err(_) => {
                eprintln!("fork() failed.");
                exitcode = EXIT_FAILURE;
                for j in 0..ncreated {
                    if let Err(_) = kill(pids[j as usize], Signal::SIGINT) {
                        eprintln!("kill({}) failed", pids[j as usize]);
                    }
                }
                break;
            }
        }
    }
    for _i in 0..ncreated {
        if let Err(_) = wait() {
            eprintln!("wait() failed.");
        }
    }
    exit(exitcode);
}
