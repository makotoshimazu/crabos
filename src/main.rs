#![no_std]
#![no_main]

use core::arch;
use core::panic::PanicInfo;
use core::ptr;

arch::global_asm!(include_str!("../boot.s"));

#[no_mangle]
pub extern "C" fn __start_rust() -> ! {
    for c in b"Hello World\n" {
        unsafe { ptr::write_volatile(0x10000000 as *mut u8, *c) };
    }

    loop {}
}

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    loop {}
}
