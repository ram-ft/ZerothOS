//! Interrupt Descriptor Table (IDT) for Zeroth OS

use core::arch::asm;

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct IdtEntry {
    offset_low: u16,
    selector: u16,
    ist: u8,
    type_attr: u8,
    offset_mid: u16,
    offset_high: u32,
    zero: u32,
}

impl IdtEntry {
    pub const fn missing() -> Self {
        Self {
            offset_low: 0,
            selector: 0,
            ist: 0,
            type_attr: 0,
            offset_mid: 0,
            offset_high: 0,
            zero: 0,
        }
    }

    pub fn set_handler(&mut self, handler: usize) {
        self.offset_low = handler as u16;
        self.selector = 0x08; // Kernel code segment
        self.ist = 0;
        self.type_attr = 0x8E; // Present, Ring 0, 64-bit Interrupt Gate
        self.offset_mid = (handler >> 16) as u16;
        self.offset_high = (handler >> 32) as u32;
        self.zero = 0;
    }
}

#[repr(C, packed)]
struct IdtDescriptor {
    limit: u16,
    base: u64,
}

static mut IDT: [IdtEntry; 256] = [IdtEntry::missing(); 256];

pub fn init_idt() {
    unsafe {
        let desc = IdtDescriptor {
            limit: (core::mem::size_of::<[IdtEntry; 256]>() - 1) as u16,
            base: (&raw const IDT) as *const _ as u64,
        };
        asm!("lidt [{}]", in(reg) &desc, options(readonly, nostack, preserves_flags));
    }
}
