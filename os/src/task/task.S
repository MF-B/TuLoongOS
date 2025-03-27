.altmacro
.macro SAVE_S n
    st.d $s\n, $a0, (\n+2)*8
.endm

.macro LOAD_S n
    ld.d $s\n, $a1, (\n+2)*8
.endm

    .section .text
    .globl __switch
    .align 2
__switch:
    # 保存当前任务上下文到current_task_cx
    st.d $ra, $a0, 0
    st.d $sp, $a0, 1*8
    .set n,0
    .rept 10
        SAVE_S %n
        .set n, n+1
    .endr
    
    # 将下一个任务的上下文切换到当前位置
    
    ld.d $ra, $a1, 0*8
    .set n,0
    .rept 10
        LOAD_S %n
        .set n, n+1
    .endr
    ld.d $sp, $a1, 1*8
    csrwr $a2,0x18
    jr $ra
