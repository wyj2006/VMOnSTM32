mov r0, #0b11111111
bfc r0, #4, #4
cmp r0, #0b00001111
bne fail