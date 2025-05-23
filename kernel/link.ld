ENTRY(_start)

SECTIONS
{
    . = 0x40100000; /* SRAM A1 */
    . = ALIGN(1);

    .text : { *(.text .text.*) }

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
