extern crate alloc;

use crate::{
    SERIAL,
    exception::Exception,
    protocol::{Command, receive_data},
};
use alloc::vec::Vec;
use bitvec::{field::BitField, order::Lsb0, view::BitView};
use core::cmp::min;
use cortex_m::interrupt;
use funty::Integral;

pub static MEM_INTERNAL_SIZE: usize = 1024 * 200;
pub static MEM_EXTERNAL_SIZE: usize = 1024 * 1024;

pub struct Memory {
    pub data: [u8; MEM_INTERNAL_SIZE],
}

impl Default for Memory {
    fn default() -> Self {
        Memory {
            data: [0; MEM_INTERNAL_SIZE],
        }
    }
}

impl Memory {
    pub fn size(&self) -> usize {
        MEM_INTERNAL_SIZE + MEM_EXTERNAL_SIZE
    }

    pub fn read<T: Integral>(&self, address: usize) -> Result<T, Exception> {
        if address >= self.size() {
            return Err(Exception::LoadAccessFault);
        }
        let mut bytes = Vec::new();

        let mut i = 0;
        while i < 4 as usize && address + i < MEM_INTERNAL_SIZE {
            bytes.push(self.data[address + i]);
            i += 1;
        }

        interrupt::free(|cs| -> Result<(), Exception> {
            if let Some(serial) = SERIAL.borrow(cs).borrow_mut().as_mut() {
                while i < 4 && address + i < MEM_EXTERNAL_SIZE {
                    Command::ReadMemory(address + i).send(serial)?;
                    bytes.extend(receive_data(serial)?);
                    i += 1;
                }
                Ok(())
            } else {
                unreachable!()
            }
        })?;

        let mut value = 0;
        for i in bytes.iter().rev() {
            value = value << 8 | *i as u32;
        }
        Ok(value.view_bits::<Lsb0>().load::<T>())
    }

    pub fn write<T: Integral + BitView>(
        &mut self,
        address: usize,
        value: T,
    ) -> Result<(), Exception> {
        if address >= self.size() {
            return Err(Exception::StoreAccessFault);
        }

        let mut bytes = Vec::new();

        let bits = T::BITS as usize;
        let value = value.view_bits::<Lsb0>();
        let mut i = 0;
        while i < bits {
            bytes.push(value.get(i..min(i + 8, bits)).unwrap().load::<u8>());
            i += 8;
        }

        i = 0;
        while i < bytes.len() && address + i < MEM_INTERNAL_SIZE {
            self.data[address + i] = bytes[i];
            i += 1;
        }

        interrupt::free(|cs| -> Result<(), Exception> {
            if let Some(serial) = SERIAL.borrow(cs).borrow_mut().as_mut() {
                while i < bytes.len() && address + i < MEM_EXTERNAL_SIZE {
                    Command::WriteMemory(address + i, bytes[i]).send(serial)?;
                    i += 1;
                }
                Ok(())
            } else {
                unreachable!()
            }
        })?;

        Ok(())
    }
}
