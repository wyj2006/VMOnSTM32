mov r0, #0b00001111
mov r1, #0b01011010
bfi r0, r1, #4, #4
cmp r0, #0b10101111
bne fail