mov r0, #1
lsl r0, #31
sub r0, #1              @ r0 = (1<<31)-1 = i32::MAX
mov r1, r0
mov r2, r0
qadd r3, r0, r1
cmp r3, r2
bne fail
mov r5, #1
lsl r5, #15
sub r5, #1              @ r5 = i16::MAX
qadd16 r3, r0, r1       @ r3 = 0b01111111_11111111_11111111_11111110
lsr r4, r3, #16
cmp r4, r5
bne fail
mov r5, #1
lsl r5, #16
sub r5, #1              @ r5 = u16::MAX
and r4 ,r3, r5
sub r5, #1              @ r5 = 0xfffe = 0b11111111_11111110
cmp r4, r5
bne fail