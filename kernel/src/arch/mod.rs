pub mod serial;
pub mod gdt;
pub mod interrupts;

use core::arch::asm;

/// Halt CPU until next interrupt (powersaving for laptops)
#[inline]
pub fn hlt_loop() -> ! {
    loop {
        unsafe {
            asm!("hlt", options(nomem, nostack, preserves_flags));
        }
    }
}
