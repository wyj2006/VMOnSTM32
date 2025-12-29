"""
单片机在准备发送数据前先发送数据0xaa, 电脑在回复0x55后才能继续传送数据
"""

import struct
import time

from command import *
from memory import *
from serial import Serial
from serial.tools.list_ports import comports
from state import *

ESCAPE_CHAR = ord("\\")
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
memory = Memory()

while True:
    print("Current state:", state)
    match state:
        case Ready():
            byte = serial.read(1)[0]
            if byte == 0xAA:
                serial.write(bytes([0x55]))
                serial.flush()
                state = ReceiveHead()
            else:
                print(f"Read {hex(byte)}({bin(byte)}), not the start of a frame")
        case ReceiveHead():
            byte = serial.read(1)[0]
            if byte in Command._value2member_map_:
                state = ReceiveData(command=Command._value2member_map_[byte])
            else:
                print("Incorrect command id:", byte)
                state = Ready()
        case ReceiveData(command, data, escape):
            byte = serial.read(1)[0]
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
                case Command.ReadMemory:
                    (address,) = struct.unpack("<I", bytes(received_data))
                    print("Address:", address)
                    data.append(memory.read(address))
                case Command.WriteMemory:
                    address, value = struct.unpack("<IB", bytes(received_data))
                    print("Address:", address)
                    print("Value:", value)
                    memory.write(address, value)
            if not data:
                state = Ready()
                continue
            i = 0
            while i < len(data):
                if data[i] in (ESCAPE_CHAR, FRAME_END):
                    data.insert(i, ESCAPE_CHAR)
                i += 1
            data.append(FRAME_END)
            data = bytes(data)
            print("Send:", data)
            serial.write(data)
            serial.flush()
            state = Ready()
