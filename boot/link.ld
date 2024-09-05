ENTRY(_start)

SECTIONS
{
    . = 0x20000; /* SRAM A1 */
    . = ALIGN(1);

    .text : { KEEP(*(.text.boot)) *(.text .text.*) }

    . = ALIGN(16);
    .rodata : { *(.rodata .rodata.* .srodata.*) }

    . = ALIGN(16);
    .data : { *(.data .data.* .sdata) }

    . = ALIGN(4);
    __bss_start = .;
    .bss : {
        bss = .;
        *(.bss .bss.*)
    }
    . = ALIGN(4);
    __bss_end = .;

    __end = .;
}
