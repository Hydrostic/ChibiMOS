.altmacro
.macro SAVE_GN n
    save s\n, \n*8(a0)
.endm
.macro LOAD_GN n
    load s\n, \n*8(a1)
.endm


    .section .text
    .globl __switch
__switch:
    # sd ra, 0(a0)
    # sd sp, 8(a0)
    # seems no need to save registers s0-s11 since they will be saved by __alltrap
    # .set n 2
    # .rept 12
    #    SAVE_GN \n
        # .set n, n+1
    # .endr
    # # load next
    ld ra, 0(a0)
    ld sp, 8(a0)
    # .set n 2
    # .rept 12
        # LOAD_GN \n
        # .set n, n+1
    # .endr
    ret


