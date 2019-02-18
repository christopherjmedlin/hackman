pub trait Memory {
    
    /// Read a byte from memory at address <addr>
    fn read_byte(&self, addr: u16) -> u8;

    /// Write byte <byte> to memory at address <addr>
    fn write_byte(&mut self, byte: u8, addr: u16);

    /// Read 16 bit word <word> at address <addr>
    fn read_word(&self, addr: u16) -> u16 {
        (self.read_byte(addr + 1) as u16) << 8 | (self.read_byte(addr) as u16)
    }

    /// Write 16 bit word <word> to address <addr>
    fn write_word(&mut self, word: u16, addr: u16) {
        self.write_byte((word >> 8) as u8, addr + 1);
        self.write_byte(word as u8, addr);
    }
}

/// An implementation of Memory trait for testing purposes
pub struct TestMemory {
    pub ram: [u8; 0x50FF]
}

impl TestMemory {
    pub fn new() -> Self {
        TestMemory {
            ram: [0; 0x50FF]
        }
    }
}

impl Memory for TestMemory {
    fn read_byte(&self, addr: u16) -> u8 {
        self.ram[addr as usize]
    }

    fn write_byte(&mut self, byte: u8, addr: u16) {
        self.ram[addr as usize] = byte;
    }
}
