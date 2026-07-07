ENTRY(start)

SECTIONS {
    . = 1M;

    .text : {
        KEEP(*(.multiboot))
        *(.text .text.*)
        . = ALIGN(8);
    }

    .rodata : {
        *(.rodata .rodata.*)
        . = ALIGN(8);
    }

    .data : {
        *(.data .data.*)
        . = ALIGN(8);
    }

    _bss_start = .;
    .bss : {
        *(COMMON)
        *(.bss .bss.*)
        . = ALIGN(8);
    }
    _bss_end = .;

    /DISCARD/ : {
        *(.eh_frame)
        *(.comment)
        *(.note.*)
    }
}
