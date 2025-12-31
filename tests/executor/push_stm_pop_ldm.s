mov r0, #1
mov r1, #2
push {r0-r1}    @STM
pop {r0-r1}     @LDM
cmp r0, #2
bne fail
cmp r1, #1
bne fail