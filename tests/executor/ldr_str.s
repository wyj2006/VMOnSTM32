mov r0, #1
str r0, [lr,#4]
mov r0, #0
ldr r0, [lr,#4]
cmp r0, #1
bne fail