extern crate alloc;
use alloc::vec::Vec;

use crate::{SerialType, vmerror::VMError};

pub const ESCAPE_CHAR: u8 = b'\\';
pub const FRAME_END: u8 = 0xff;

pub enum Command {
    ReadMemory(u32),
}

impl Command {
    pub fn head(&self) -> u8 {
        match self {
            Command::ReadMemory(..) => 1,
        }
    }

    pub fn data(&self) -> Vec<u8> {
        match self {
            Command::ReadMemory(address) => address.to_le_bytes().to_vec(),
        }
    }

    pub fn send(&self, serial: &mut SerialType) -> Result<(), VMError> {
        ensure_ready(serial)?;
        serial.tx.write_u8(self.head())?;
        for i in self.data() {
            if i == ESCAPE_CHAR || i == FRAME_END {
                serial.tx.write_u8(ESCAPE_CHAR)?;
            }
            serial.tx.write_u8(i)?;
        }
        serial.tx.write_u8(FRAME_END)?;
        serial.tx.flush()?;
        Ok(())
    }
}

pub fn ensure_ready(serial: &mut SerialType) -> Result<(), VMError> {
    serial.tx.write_u8(0)?;
    serial.tx.flush()?;
    serial.rx.read()?;
    Ok(())
}

pub fn receive_data(serial: &mut SerialType) -> Result<Vec<u8>, VMError> {
    let mut data = Vec::new();
    let mut escape = false;
    loop {
        let byte = serial.rx.read()?;
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
