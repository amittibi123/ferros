ENTRY(kmain)

PHDRS
{
    text   PT_LOAD FLAGS(5); /* Read + Execute */
    rodata PT_LOAD FLAGS(4); /* Read-only */
    data   PT_LOAD FLAGS(6); /* Read + Write */
}

SECTIONS
{
    . = 0xffffffff80000000;

    .text : {
        *(.text .text.*)
    } :text

    . = ALIGN(0x1000);

    .rodata : {
        *(.rodata .rodata.*)
    } :rodata

    . = ALIGN(0x1000);

    .data : {
        *(.requests_start_marker)
        *(.requests)
        *(.requests_end_marker)
        *(.data .data.*)
    } :data

    .bss : {
        *(.bss .bss.*)
        *(COMMON)
    } :data

    /DISCARD/ : {
        *(.eh_frame)
        *(.note .note.*)
        *(.interp)
        *(.dynamic)
    }
}