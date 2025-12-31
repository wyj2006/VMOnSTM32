mov r0, #0b10101010
mov r1, #0b01011010
eor r0, r1
cmp r0, #0xf0
bne fail