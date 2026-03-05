addi x1,x1,1	#x1= 1
addi x2,x2,1	#x2 = 1
loop:
add x3,x1,x2	#x3=x1+x2
addi x1,x2,0	#x1 = x2
addi x2,x3,0	#x2 = x3
jal x0,loop
