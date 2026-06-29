import struct
import time

from serial import *
from serial.tools.list_ports import *

READ_MEM_CODE = 0
WRITE_MEM_CODE = 1
FRAME_START = 0xAA
FRAME_END = 0x55

port_name = None
while port_name == None:
    print("Automatically find the correct port...", end="")
    for port in comports():
        if "UART" in port.description:
            port_name = port.name
            break
    else:
        time.sleep(1)
        print(end="\r")
print(port_name)

memory = {}
data = []
serial = Serial(port_name, 115200)

while True:
    byte = serial.read(1)[0]
    if byte != FRAME_START:
        continue
    while byte != FRAME_END:
        data.append(byte)
        byte = serial.read(1)[0]
        while byte == ord("\\"):
            byte = serial.read(1)[0]

    if data[1] == READ_MEM_CODE:
        (address,) = struct.unpack("<I", bytes(data[2:]))
        if address not in memory:
            memory[address] = 0
        serial.write(memory[address])
        serial.flush()
    elif data[1] == WRITE_MEM_CODE:
        address, value = struct.unpack("<IB", bytes(data[2:]))
        memory[address] = value
