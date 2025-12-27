"""
单片机在准备发送数据前先发送数据, 电脑在回复后才能继续传送数据
"""

import struct
import time

from serial import Serial
from serial.tools.list_ports import comports
from state import *

ESCAPE_CHAR = "\\"
FRAME_END = 0xFF

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

state = Ready()
serial = Serial(port_name, 115200)

while True:
    match state:
        case Ready():
            serial.read(1)
            serial.write(b"0")
            serial.flush()
            state = ReceiveHead()
        case ReceiveHead():
            state = ReceiveData(command=serial.read(1))
        case ReceiveData(command, data, escape):
            byte = serial.read(1)
            if not escape and byte == ESCAPE_CHAR:
                state.escape = True
            elif not escape and byte == FRAME_END:
                state = Process(command, data)
            else:
                state.escape = False
                data.append(byte)
        case Process(command, received_data):
            data = []
            match command:
                case 1:  # ReadMemory
                    print(received_data)
                    address = struct.unpack("<I", received_data)[0]
                    data.append(struct.pack("<I", 0xE2800001)[address % 4 - 1])
            if not data:
                state = Ready()
                continue
            for i in data:
                if data in (ESCAPE_CHAR, FRAME_END):
                    serial.write(ESCAPE_CHAR)
                serial.write(i)
            serial.flush()
