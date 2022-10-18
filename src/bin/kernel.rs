#![no_std]
#![no_main]
#![feature(abi_efiapi)]

use core::arch::asm;
use core::panic::PanicInfo;
use core::slice::from_raw_parts_mut;

#[no_mangle]
pub extern "efiapi" fn kernel_main(frame_buffer_base: *mut u8, frame_buffer_size: u64) -> ! {
    let frame_buffer = unsafe { from_raw_parts_mut(frame_buffer_base, frame_buffer_size as usize) };

    // こっちの方がRustっぽい
    for (i, v) in frame_buffer.iter_mut().enumerate() {
        *v = (i % 256) as u8;
    }
    // method chainするならこう
    // frame_buffer.iter_mut().enumerate().for_each(|(i, v)|)
    // mapだけだとIteratorになるので実行されない (書き換えが起らない)
    // frame_buffer.iter_mut().enumerate().map(|(i, v)|)
    // やるならこうか？
    // frame_buffer.iter_mut().enumerate().map(|(i, v)|).collect()
    loop {
        unsafe {
            asm!("hlt");
        }
    }
}

#[panic_handler]
fn panic(_panic: &PanicInfo<'_>) -> ! {
    loop {}
}
