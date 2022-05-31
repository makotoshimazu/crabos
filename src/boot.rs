core::arch::global_asm!(include_str!("boot.s"));

#[no_mangle]
pub extern "C" fn __start_rust() -> ! {
    let uart = 0x1000_0000 as *mut u8;
    for c in b"Hello from Rust!".iter() {
        unsafe {
            *uart = *c as u8;
        }
    }

    crate::main()
}

use core::panic::PanicInfo;
#[panic_handler]
#[no_mangle]
pub fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn abort() -> ! {
    loop {}
}
