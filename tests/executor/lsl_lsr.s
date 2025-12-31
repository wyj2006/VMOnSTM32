mov r0, #1
lsl r0, r0, #10
cmp r0, #1024
bne fail
lsr r0, r0, #10
cmp r0, #1
bne fail