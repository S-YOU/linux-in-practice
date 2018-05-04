extern crate nix;

fn main() {
    loop {
        nix::unistd::getppid();
    }
}
