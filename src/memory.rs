use crate::machine::Machine;

const INTERNAL_SIZE: usize = 1024;
const EXTERNAL_SIZE: usize = 1024;

pub struct Memory {
    pub data: [u8; INTERNAL_SIZE],
}

impl Default for Memory {
    fn default() -> Self {
        Memory {
            data: {
                let mut data = [0; INTERNAL_SIZE];
                /*
                loop:
                add r0,#1   ; e2800001
                B loop      ; eafffffc
                */
                data[0] = 0x01;
                data[1] = 0x00;
                data[2] = 0x80;
                data[3] = 0xe2;
                data[4] = 0xfc;
                data[5] = 0xff;
                data[6] = 0xff;
                data[7] = 0xea;
                data
            },
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

    pub fn read_memory_n(&self, address: u32, buf: &mut [u8]) {
        for i in 0..buf.len() {
            buf[i] = self.read_memory(address + i as u32);
        }
    }

    pub fn read_memory_halfword(&self, address: u32) -> u16 {
        let mut word_bytes: [u8; _] = [0; 2];
        self.read_memory_n(address, &mut word_bytes);
        u16::from_le_bytes(word_bytes)
    }

    pub fn read_memory_word(&self, address: u32) -> u32 {
        let mut word_bytes: [u8; _] = [0; 4];
        self.read_memory_n(address, &mut word_bytes);
        u32::from_le_bytes(word_bytes)
    }

    pub fn write_memory(&mut self, address: u32, bit: u8) {
        let address = address as usize;
        if address < INTERNAL_SIZE {
            self.memory.data[address] = bit
        } else {
            unimplemented!()
        }
    }

    pub fn write_memory_n(&mut self, address: u32, buf: &[u8]) {
        for i in 0..buf.len() {
            self.write_memory(address + i as u32, buf[i]);
        }
    }

    pub fn write_memory_halfword(&mut self, address: u32, halfword: u16) {
        self.write_memory_n(address, &halfword.to_le_bytes());
    }

    pub fn write_memory_word(&mut self, address: u32, word: u32) {
        self.write_memory_n(address, &word.to_le_bytes());
    }
}
