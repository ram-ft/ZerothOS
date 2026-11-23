/* Linker script for Zeroth OS kernel (x86_64) */
ENTRY(_start)

OUTPUT_FORMAT(elf64-x86-64)
OUTPUT_ARCH(i386:x86-64)

PHDRS
{
    text    PT_LOAD FLAGS((1 << 0) | (1 << 2)); /* Execute + Read */
    rodata  PT_LOAD FLAGS((1 << 2));            /* Read only */
    data    PT_LOAD FLAGS((1 << 1) | (1 << 2)); /* Write + Read */
}

SECTIONS
{
    /* Higher-half base address for 64-bit kernel */
    . = 0xffffffff80000000;

    .text : {
        *(.text .text.*)
    } :text

    . = ALIGN(4096);

    .rodata : {
        *(.rodata .rodata.*)
    } :rodata

    . = ALIGN(4096);

    .data : {
        *(.requests_start_marker)
        *(.requests)
        *(.requests_end_marker)
        *(.data .data.*)
    } :data

    . = ALIGN(4096);

    .bss : {
        *(COMMON)
        *(.bss .bss.*)
    } :data

    /DISCARD/ : {
        *(.eh_frame)
        *(.note .note.*)
    }
}
