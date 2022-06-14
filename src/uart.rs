use core::fmt;
use core::ptr;

pub struct Uart(*mut u8);

impl Uart {
    pub fn new() -> Self {
        Self(0x10000000 as _)
    }
}

impl fmt::Write for Uart {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.bytes() {
            unsafe { ptr::write_volatile(self.0, c) };
        }
        Ok(())
    }
}
