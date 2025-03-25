BASE_ADDRESS = 0x10000;
OUTPUT_ARCH(loongarch)
ENTRY(_start)

SECTIONS
{
    . = BASE_ADDRESS;
    s_text = .;
    .text : {
        *(.text.entry)
        *(.text .text.*)
    }
    . = ALIGN(16K);
    e_text = .;
    s_rodata = .;
    .rodata : {
        *(.rodata .rodata.*)
        *(.srodata .srodata.*)
    }
    . = ALIGN(16K);
    e_rodata = .;
    s_data = .;
    .data : {
        *(.data .data.*)
        *(.sdata .sdata.*)
    }
    e_data = .;
    .bss : {
        s_bss = .;
        *(.bss .bss.*)
        *(.sbss .sbss.*)
        e_bss = .;
    }
    /DISCARD/ : {
        *(.eh_frame)
        *(.debug*)
    }
}
