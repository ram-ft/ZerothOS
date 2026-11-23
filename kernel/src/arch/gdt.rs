//! Global Descriptor Table (GDT) for Zeroth OS (x86_64)
//! Limine Bootloader protocol loads 64-bit long mode with valid CS and DS segments.

pub fn init_gdt() {
    // Limine provides a verified 64-bit Long Mode GDT at entry.
    // We register our confirmation here.
}
