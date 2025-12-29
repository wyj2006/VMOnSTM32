extern crate alloc;
use crate::{serial::ProtocolSerial, vmerror::VMError};
use alloc::vec::Vec;

pub const ESCAPE_CHAR: u8 = b'\\';
pub const FRAME_END: u8 = 0xff;

pub enum Command {
    ReadMemory(u32),
    WriteMemory(u32, u8),
}

impl Command {
    pub fn head(&self) -> u8 {
        match self {
            Command::ReadMemory(..) => 1,
            Command::WriteMemory(..) => 2,
        }
    }

    pub fn data(&self) -> Vec<u8> {
        match self {
            Command::ReadMemory(address) => address.to_le_bytes().to_vec(),
            Command::WriteMemory(address, value) => {
                let mut data = address.to_le_bytes().to_vec();
                data.extend(value.to_le_bytes().to_vec());
                data
            }
        }
    }

    pub fn send(&self, serial: &mut ProtocolSerial) -> Result<(), VMError> {
        ensure_ready(serial)?;
        serial.write(self.head())?;
        for i in self.data() {
            if i == ESCAPE_CHAR || i == FRAME_END {
                serial.write(ESCAPE_CHAR)?;
            }
            serial.write(i)?;
        }
        serial.write(FRAME_END)?;
        serial.flush()?;
        Ok(())
    }
}

pub fn ensure_ready(serial: &mut ProtocolSerial) -> Result<(), VMError> {
    loop {
        serial.write(0xaa)?;
        serial.flush()?;
        if serial.read()? == 0x55 {
            break;
        }
    }
    Ok(())
}

pub fn receive_data(serial: &mut ProtocolSerial) -> Result<Vec<u8>, VMError> {
    let mut data = Vec::new();
    let mut escape = false;
    loop {
        let byte = serial.read()?;
        if !escape && byte == ESCAPE_CHAR {
            escape = true;
        } else if !escape && byte == FRAME_END {
            break;
        } else {
            escape = false;
            data.push(byte);
        }
    }
    Ok(data)
}
