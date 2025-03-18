.altmacro
.macro SAVE_GP n
    st.d $r\n, $sp, \n*8
.endm

.macro LOAD_GP n
    ld.d $r\n, $sp, \n*8
.endm

    .section .text
    .globl __restore
    .globl __alltraps
    .align 2
__alltraps:
    # 将sp的值更新到0x30上,同时将0x30:SAVE的值保存到sp上
    # sp->kernel stack, 0x30 -> user stack
    csrwr $sp, 0x30

    # 将TrapContext保存在kernel stack
    # 分配空间
    addi.d $sp, $sp, -34*8

    # 保存通用寄存器
    st.d $r1, $sp, 1*8
    # 跳过tp和sp
    # 保存r4~r20
    .set n, 4
    .rept 17
        SAVE_GP %n
        .set n, n+1
    .endr
    # 跳过r21
    # 保存 r22-r31
    .set n, 22
    .rept 10
        SAVE_GP %n
        .set n, n+1
    .endr

    # 可以随便使用 t0/t1/t2 寄存器了, 因为原值已存入kernel stack
    # 将era保存到t1(龙芯架构发生例外(中断)时,crmd会自动保存到prmd,故不需要保存crmd)
    csrrd $t1, 0x6
    st.d $t1, $sp, 32*8
    csrrd $t0, 0x1
    st.d $t0, $sp, 33*8

    # 将user stack的sp指针存入kernel stack
    # 至此user程序的所有寄存器保存完成
    csrrd $t2, 0x30
    st.d $t2, $sp, 3*8

    # 将kernal stack的sp指针设置为trap_handler的参数,让trap_handler能根据应用程序的寄存器值来判断例外类型
    # 执行完trap_handler后,返回user程序的系统调用syscall的后一条指令(jr $ra),而ra寄存器在bl tarp_handler时被保存为bl指令的下一条指令,及__restore
    move $a0, $sp
    bl trap_handler

__restore:
    # case1: start running app by __restore
    # case2: back to U after handling trap
    # 将内核栈指针保存到0x30
    # csrwr $sp, 0x30
    # csrrd $sp, 0x30

    # 将当前指针指向cx
    move $sp, $a0

    # now sp->kernel stack(after allocated), sscratch->user stack
    # restore era
    
    ld.d $t1, $sp, 32*8
    ld.d $t2, $sp, 3*8
    ld.d $t0, $sp, 33*8
    csrwr $t1, 0x6
    csrwr $t2, 0x30
    csrwr $t0, 0x1

    # restore general-purpuse registers except sp/tp
    ld.d $r1, $sp, 1*8
    .set n, 4
    .rept 17
        LOAD_GP %n
        .set n, n+1
    .endr

    .set n, 22
    .rept 10
        LOAD_GP %n
        .set n, n+1
    .endr

    # release TrapContext on kernel stack
    addi.d $sp, $sp, 34*8

    # 让 0x30->kernel stack, sp->user stack
    csrwr $sp, 0x30
    ertn