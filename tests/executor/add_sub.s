mov r0, #0
add r0, #1
cmp r0, #1
bne fail
add r0, #-2             @sub
cmp r0, #-1
bne fail