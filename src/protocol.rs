extern crate alloc;

use crate::{ProtocolSerial, exception::Exception};
use alloc::vec::Vec;
use cortex_m::prelude::{_embedded_hal_serial_Read, _embedded_hal_serial_Write};

pub const ESCAPE_CHAR: u8 = b'\\';
pub const FRAME_END: u8 = 0xff;
pub const READY_INQ: u8 = 0xaa;
pub const READY_ACK: u8 = 0x55;
pub const RECV_ACK: u8 = 0xa5;

pub enum Command {
    ReadMemory(usize),
    WriteMemory(usize, u8),
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

    pub fn send(&self, serial: &mut ProtocolSerial) -> Result<(), Exception> {
        loop {
            serial.write(READY_INQ)?;
            serial.flush()?;
            if serial.read()? == READY_ACK {
                break;
            }
        }

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

pub fn receive_data(serial: &mut ProtocolSerial) -> Result<Vec<u8>, Exception> {
    let mut data = Vec::new();
    let mut escape = false;
    loop {
        serial.write(RECV_ACK)?;
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
