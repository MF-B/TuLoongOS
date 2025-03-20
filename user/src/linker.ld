BASE_ADDRESS = 0x800000;
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
    e_text = .;
    s_rodata = .;
    .rodata : {
        *(.rodata .rodata.*)
        *(.srodata .srodata.*)
    }
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
