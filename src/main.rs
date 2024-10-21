#![no_main]
#![no_std]

use core::panic::PanicInfo;
use core::alloc::{GlobalAlloc, Layout};
use core::ptr::null_mut;
use core::sync::atomic::{AtomicUsize, Ordering};



#[panic_handler]
fn panic(_panic: &PanicInfo) -> ! {
    loop {}
}