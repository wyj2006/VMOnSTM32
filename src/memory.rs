extern crate alloc;

use crate::{
    PANIC_PIN, SERIAL,
    cpu::{Mode, SATP},
    machine::Machine,
    protocol::{Command, receive_data},
    vm_exception::VMException,
};
use alloc::vec::Vec;
use bitfield::bitfield;
use core::ops::{BitAnd, BitOr, Shl, Shr};
use cortex_m::interrupt;
use funty::Integral;

pub static MEM_INTERNAL_SIZE: usize = 1024 * 100;
pub static MEM_EXTERNAL_SIZE: usize = 1024 * 1024;
pub static MEM_PERIPH_SIZE: usize = 1;

pub trait AsCast<T> {
    fn r#as(&self) -> T;
}

macro_rules! impl_as_cast {
    ($from:ty as $to:ty) => {
        impl AsCast<$to> for $from {
            fn r#as(&self) -> $to {
                *self as $to
            }
        }
    };
}

impl_as_cast!(u8 as u8);
impl_as_cast!(u8 as u16);
impl_as_cast!(u8 as u32);
impl_as_cast!(u8 as u64);

impl_as_cast!(u8 as i8);
impl_as_cast!(u8 as i16);
impl_as_cast!(u8 as i32);
impl_as_cast!(u8 as i64);

impl_as_cast!(u16 as u8);

impl_as_cast!(u32 as u8);

impl_as_cast!(u64 as u8);
impl_as_cast!(u64 as u16);
impl_as_cast!(u64 as u32);
impl_as_cast!(u64 as u64);

impl_as_cast!(u64 as i8);
impl_as_cast!(u64 as i16);
impl_as_cast!(u64 as i32);

bitfield! {
    pub struct PageTableEntry(u32);

    pub v,set_v:0;
    pub r,set_r:1;
    pub w,set_w:2;
    pub x,set_x:3;
    pub u,set_u:4;
    pub g,set_g:5;
    pub a,set_a:6;
    pub d,set_d:7;
    pub rsw,set_rsw:9,8;
    pub ppn,set_ppn:31,10;
}

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
        MEM_INTERNAL_SIZE + MEM_EXTERNAL_SIZE + MEM_PERIPH_SIZE
    }

    pub fn read<T>(&self, address: usize) -> Result<T, VMException>
    where
        T: Integral + Shl + BitOr,
        u64: AsCast<T>,
        u8: AsCast<T>,
    {
        if address >= self.size() || address + T::BITS.div_ceil(8) as usize > self.size() {
            return Err(VMException::LoadAccessFault);
        }
        let mut bytes = Vec::new();

        let mut i = 0;
        while i < T::BITS.div_ceil(8) as usize && address + i < MEM_INTERNAL_SIZE {
            bytes.push(self.data[address + i]);
            i += 1;
        }

        interrupt::free(|cs| -> Result<(), VMException> {
            if let Some(serial) = SERIAL.borrow(cs).borrow_mut().as_mut() {
                while i < T::BITS.div_ceil(8) as usize
                    && address + i < MEM_INTERNAL_SIZE + MEM_EXTERNAL_SIZE
                {
                    Command::ReadMemory(address + i).send(serial)?;
                    bytes.extend(receive_data(serial)?);
                    i += 1;
                }
                Ok(())
            } else {
                unreachable!()
            }
        })?;

        while i < T::BITS.div_ceil(8) as usize
            && address + i < MEM_INTERNAL_SIZE + MEM_EXTERNAL_SIZE + MEM_PERIPH_SIZE
        {
            match address + i - MEM_INTERNAL_SIZE - MEM_EXTERNAL_SIZE {
                0 => interrupt::free(|cs| -> Result<(), VMException> {
                    if let Some(pin) = PANIC_PIN.borrow(cs).borrow_mut().as_ref() {
                        bytes.push(pin.is_set_high() as u8);
                        Ok(())
                    } else {
                        unreachable!()
                    }
                })?,
                _ => unreachable!(),
            }
            i += 1;
        }

        let mut value: T = T::ZERO;
        for i in bytes.iter().rev() {
            value = value << 8 | (*i).r#as();
        }
        Ok(value)
    }

    pub fn write<T>(&mut self, address: usize, value: T) -> Result<(), VMException>
    where
        T: Integral + Shr + BitAnd + AsCast<u8>,
        u8: AsCast<T>,
    {
        if address >= self.size() || address + T::BITS.div_ceil(8) as usize > self.size() {
            return Err(VMException::StoreAccessFault);
        }

        let mut bytes: Vec<u8> = Vec::new();

        let bits = T::BITS as usize;
        let mut i = 0;
        while i < bits {
            bytes.push((value >> i & 0xff.r#as()).r#as());
            i += 8;
        }

        i = 0;
        while i < bytes.len() && address + i < MEM_INTERNAL_SIZE {
            self.data[address + i] = bytes[i];
            i += 1;
        }

        interrupt::free(|cs| -> Result<(), VMException> {
            if let Some(serial) = SERIAL.borrow(cs).borrow_mut().as_mut() {
                while i < bytes.len() && address + i < MEM_INTERNAL_SIZE + MEM_EXTERNAL_SIZE {
                    Command::WriteMemory(address + i, bytes[i]).send(serial)?;
                    i += 1;
                }
                Ok(())
            } else {
                unreachable!()
            }
        })?;

        while i < bytes.len()
            && address + i < MEM_INTERNAL_SIZE + MEM_EXTERNAL_SIZE + MEM_PERIPH_SIZE
        {
            match address + i - MEM_INTERNAL_SIZE - MEM_EXTERNAL_SIZE {
                0 => interrupt::free(|cs| -> Result<(), VMException> {
                    if let Some(pin) = PANIC_PIN.borrow(cs).borrow_mut().as_mut() {
                        if bytes[i] == 0 {
                            pin.set_low();
                        } else {
                            pin.set_high();
                        }
                        Ok(())
                    } else {
                        unreachable!()
                    }
                })?,
                _ => unreachable!(),
            }
            i += 1;
        }

        Ok(())
    }
}

impl Machine {
    //如果需要转换地址, 返回转换后的地址, 否则返回原地址
    pub fn translate_address(&self, address: usize) -> Result<usize, VMException> {
        Ok(match self.cpu.mode {
            Mode::Machine => address,
            _ => {
                let satp = SATP(self.cpu.satp());
                if satp.mode() == false {
                    address
                } else {
                    let vpn1 = address >> 22;
                    let vpn0 = address >> 12 & 0x3ff;
                    let offset = address & 0xfff;
                    let ppn = satp.ppn() as usize;

                    let pte = PageTableEntry(self.memory.read::<u32>(ppn * 4096 + vpn1 * 4)?);
                    let pte = PageTableEntry(
                        self.memory
                            .read::<u32>(pte.ppn() as usize * 4096 + vpn0 * 4)?,
                    );
                    pte.ppn() as usize * 4096 + offset
                }
            }
        })
    }
}
