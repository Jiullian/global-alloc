#![no_main]
#![no_std]

use core::panic::PanicInfo;
use core::alloc::{GlobalAlloc, Layout};
use core::cell::UnsafeCell;
use core::ptr::null_mut;
extern crate alloc;
use alloc::boxed::Box;

//Définir la structure de l'allocateur
struct MonAllocateur{
    next: UnsafeCell<usize>,
}

//Implémentation de Sync afin que l'allocateur puisse être partagé entre plusieurs threads sans soucis
unsafe impl Sync for MonAllocateur{}

//Création de la méthode new()
impl MonAllocateur{
    pub const fn new() -> Self{
        MonAllocateur{
            next: UnsafeCell::new(0),
        }
    }
}

//Implémentation du trait GlobalAlloc
unsafe impl GlobalAlloc for MonAllocateur {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let size = layout.size();
        let align = layout.align();
        let next = self.next.get();

        //Aligner le pointeur
        let mut alloc_start = *next;
        let align_offset = alloc_start % align;
        if align_offset != 0 {
            alloc_start += align - align_offset;
        }

        let alloc_end = alloc_start + size;

        //Vérifier si on a assez de mémoire
        if alloc_end > HEAP_SIZE{
            return null_mut();
        }else {
            *next = alloc_end;
            unsafe { HEAP.as_ptr().add(alloc_start) as *mut u8 }
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout){

    }
}

//Définir un espace mémoire fixe (1 Mo)
const HEAP_SIZE: usize = 1024 * 1024;
static mut HEAP: [u8; HEAP_SIZE] = [0; HEAP_SIZE];

#[global_allocator]
static ALLOCATOR: MonAllocateur = MonAllocateur::new();

#[no_mangle]
pub extern "C" fn _start() -> !{
    let x = Box::new(42);

    loop {

    }
}

#[panic_handler]
fn panic(_panic: &PanicInfo) -> ! {
    loop {}
}