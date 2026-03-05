import os
import subprocess
import sys

filepath = sys.argv[1]
filename, ext = os.path.splitext(filepath)


def run(*args, stdout=None):
    subprocess.run(
        args,
        check=True,
        stdout=stdout,
        stderr=subprocess.STDOUT,
        text=True,
        universal_newlines=True,
    )


match ext:
    case ".s":
        run(
            "riscv32-unknown-elf-as",
            filepath,
            "-o",
            f"{filename}.o",
            "-march=rv32i",
        )
    case ".c":
        run(
            "riscv32-unknown-elf-gcc",
            filepath,
            "-o",
            f"{filename}.o",
            "-c",
            "-march=rv32i",
            "-mabi=ilp32",
        )

run(
    "riscv32-unknown-elf-ld",
    f"{filename}.o",
    "-o",
    f"{filename}.elf",
)

run(
    "riscv32-unknown-elf-objcopy",
    "-O",
    "binary",
    f"{filename}.elf",
    f"{filename}.bin",
)
