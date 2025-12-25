use crate::machine::Machine;

const INTERNAL_SIZE: usize = 1024;
const EXTERNAL_SIZE: usize = 1024;

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
    pub fn read_memory(&self, address: u32) -> u8 {
        let address = address as usize;
        if address < INTERNAL_SIZE {
            self.memory.data[address]
        } else {
            unimplemented!()
        }
    }
}
