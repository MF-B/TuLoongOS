.section .text
.globl __syscall
.align 4
__syscall:
        move $t0, $a0
        move $t1, $a1
        move $t2, $a2
        move $t3, $a3
        move $a7, $t0
        move $a0, $t1
        move $a1, $t2
        move $a2, $t3
        syscall 0
        jr $ra
