import os

os.chdir(os.path.dirname(__file__))
test_code = f"mov sp, #{1024*50}\n"

for i, file_name in enumerate(os.listdir("executor")):
    file_path = os.path.join("executor", file_name)
    test_code += f"test_{os.path.splitext(file_name)[0]}:\n"
    test_code += f"mov r8, #{i}\n"  # r8存放当前测试编号
    test_code += open(file_path, encoding="utf-8").read()
    test_code += "\n\n"

# r9表示测试是否通过
test_code += """
success:
    mov r9, #1
    nop
    b success
fail:
    mov r9, #0
    nop
    b fail
"""

open("test.s", mode="w", encoding="utf-8").write(test_code)
os.system("arm-none-eabi-as -mcpu=cortex-a7 -g test.s -o test.o")
os.system("arm-none-eabi-ld test.o -o test.elf")
os.system("arm-none-eabi-objcopy -O binary test.elf test.bin")
