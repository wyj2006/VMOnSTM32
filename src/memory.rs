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
