mov r0, #0xffffffff     @mvn
mov r1, #0
adds r0, #1
adc r1, #0
cmp r0, #0
bne fail
cmp r1, #1
bne fail