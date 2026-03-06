import os
import subprocess
import sys

arch = "rv32imfd"
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
            f"-march={arch}",
        )
    case ".c":
        run(
            "riscv32-unknown-elf-gcc",
            filepath,
            "-o",
            f"{filename}.o",
            "-c",
            f"-march={arch}",
            "-mabi=ilp32d",
        )

run(
    "riscv32-unknown-elf-ld",
    f"{filename}.o",
    "-o",
    f"{filename}.elf",
)
