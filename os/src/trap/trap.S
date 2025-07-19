
.altmacro
# Important to add line above
.macro SAVE_GP n
    sd x\n, \n*8(sp)
.endm
.macro LOAD_GP n
    ld x\n, \n*8(sp)
.endm
    .section .text.trampoline
    .globl __alltraps
    .globl __restore
    .align 2
# Used cs registers:
# stvec: trap vector base address
# sscratch: scratch register for trap handling
# sstatus: status register for trap handling, we mainly use the mode field in it


# Used cs instructionsL
# csrr 1: GPR 2: CSR Read 2's value and write it to 1
# csrw 1: CSR 2: GPR Write 2's value to 1
# csrrw 1: GPR 2: CSR 3: GPR Read 2's value and write it to 1, then write 3's value to 2
# According to RISC-V ISA Manual Volume 2 10.1.2, the BASE field of `stvec` should be 4-byte aligned.
__alltraps:
    # Accordint to RISC-V ISA Manual Volume 1 7.1, unless specified otherwise, the full consequences of any such side effects are observable by the very next instruction.
    # In last restore, sscratch has been set to the kernel stack pointer(namely the address of TrapContext)
    
    csrrw sp, sscratch, sp # save current in sscratch temporarily
    # addi sp, sp, -37*8
    # ======== Start to save GPR ========
    sd x1, 1*8(sp)
    sd x3, 3*8(sp)
    .set n,5
    .rept 27
        SAVE_GP %n
        .set n, n+1
    .endr
    # ======== END =======
    # ======== Start to save CSR ========
    csrr t0, sstatus
    csrr t1, sepc
    sd t0, 32*8(sp)
    sd t1, 33*8(sp)
    csrr t2, sscratch
    sd t2, 2*8(sp)
    # a0-1 can use as the function arguments
    # mv a0, sp
    ld t0, 34*8(sp) # Load kernel satp
    ld t1, 36*8(sp) # Load kernel trap_handler entry
    ld sp, 35*8(sp) # Load kernel sp
    # ld sp, 35*8(sp) # Load kernel sp
    csrw satp, t0
    sfence.vma
    jr t1

__restore:
    # mv sp, a0
    # Switch to user address space
    csrw satp, a1
    sfence.vma

    csrw sscratch, a0
    mv sp, a0 # sp of user kernel stack
    # ======== Start to restore CSR ========
    ld t0, 32*8(sp)
    ld t1, 33*8(sp)
    csrw sstatus, t0
    csrw sepc, t1
    # ======== END ========
    ld x1, 1*8(sp)
    ld x3, 3*8(sp)
    .set n,5
    .rept 27
        LOAD_GP %n
        .set n, n+1
    .endr
    ld sp, 2*8(sp)
    # addi sp, sp, 37*8
    # csrrw sp, sscratch, sp
    sret
