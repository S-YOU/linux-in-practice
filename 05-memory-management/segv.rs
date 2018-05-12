use std::ptr;

fn main() {
    let p: *mut i32 = ptr::null_mut();
    println!("before invalid access");
    unsafe {
        *p = 0;
    }
    println!("after invalid access");
}
