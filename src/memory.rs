use cortex_m::interrupt;

use crate::{
    SERIAL,
    machine::Machine,
    protocol::{Command, receive_data},
    vmerror::VMError,
};

const INTERNAL_SIZE: usize = 1024 * 100;
const EXTERNAL_SIZE: usize = 1024 * 1024;

pub struct Memory {
    pub data: [u8; INTERNAL_SIZE],
}

impl Default for Memory {
    fn default() -> Self {
        Memory {
            data: [0; INTERNAL_SIZE],
        }
    }
}

impl Memory {
    pub fn size(&self) -> usize {
        INTERNAL_SIZE + EXTERNAL_SIZE
    }
}

impl Machine {
    pub fn read_memory(&self, address: u32) -> Result<u8, VMError> {
        let address = address as usize;
        if address >= self.memory.size() {
            Err(VMError::BusError)
        } else if address < INTERNAL_SIZE {
            Ok(self.memory.data[address])
        } else {
            interrupt::free(|cs| -> Result<u8, VMError> {
                if let Some(serial) = SERIAL.borrow(cs).borrow_mut().as_mut() {
                    Command::ReadMemory((address - INTERNAL_SIZE) as u32).send(serial)?;
                    Ok(receive_data(serial)?[0])
                } else {
                    unreachable!()
                }
            })
        }
    }

    pub fn read_memory_n(&self, address: u32, buf: &mut [u8]) -> Result<(), VMError> {
        for i in 0..buf.len() {
            buf[i] = self.read_memory(address + i as u32)?;
        }
        Ok(())
    }

    pub fn read_memory_halfword(&self, address: u32) -> Result<u16, VMError> {
        let mut word_bytes: [u8; _] = [0; 2];
        self.read_memory_n(address, &mut word_bytes)?;
        Ok(u16::from_le_bytes(word_bytes))
    }

    pub fn read_memory_word(&self, address: u32) -> Result<u32, VMError> {
        let mut word_bytes: [u8; _] = [0; 4];
        self.read_memory_n(address, &mut word_bytes)?;
        Ok(u32::from_le_bytes(word_bytes))
    }

    pub fn write_memory(&mut self, address: u32, bit: u8) -> Result<(), VMError> {
        let address = address as usize;
        if address >= self.memory.size() {
            return Err(VMError::BusError);
        }
        if address < INTERNAL_SIZE {
            self.memory.data[address] = bit
        } else {
            unimplemented!()
        }
        Ok(())
    }

    pub fn write_memory_n(&mut self, address: u32, buf: &[u8]) -> Result<(), VMError> {
        for i in 0..buf.len() {
            self.write_memory(address + i as u32, buf[i])?;
        }
        Ok(())
    }

    pub fn write_memory_halfword(&mut self, address: u32, halfword: u16) -> Result<(), VMError> {
        self.write_memory_n(address, &halfword.to_le_bytes())?;
        Ok(())
    }

    pub fn write_memory_word(&mut self, address: u32, word: u32) -> Result<(), VMError> {
        self.write_memory_n(address, &word.to_le_bytes())?;
        Ok(())
    }
}
