b bl_blx_start
bl_blx_f:
    mov r0, #1
    push {r0}
    mov r0, #2
    blx lr
bl_blx_start:
    bl bl_blx_f
    pop {r0}
    cmp r0, #1
    bne fail