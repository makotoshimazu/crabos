#![no_std]
#![no_main]

mod uart;

use core::arch;
use core::fmt::Write;
use core::panic::PanicInfo;
use uart::Uart;

arch::global_asm!(include_str!("../boot.s"));

#[no_mangle]
pub extern "C" fn __start_rust() -> ! {
    let mut uart = Uart::new();
    writeln!(&mut uart, "Hello World").unwrap();

    loop {}
}

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    loop {}
}
