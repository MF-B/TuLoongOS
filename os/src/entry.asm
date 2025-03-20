# os/src/entry.asm
    .section .text.entry
    .globl _start
_start:
    addi.d $t0,$r0,0x6
    csrwr $t0,0x2
    la.pcrel $sp, boot_stack_top
    bl rust_main

    .section .bss.stack
    .global boot_stack_lower_bound
boot_stack_lower_bound:
    .space 4096 * 16
    .globl boot_stack_top
boot_stack_top:
