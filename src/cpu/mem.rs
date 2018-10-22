pub trait Memory {
    
    /// Read a byte from memory at address <addr>
    fn read_byte(&self, addr: u16) -> u8;

    /// Write byte <byte> to memory at address <addr>
    fn write_byte(&mut self, byte: u8, addr: u16);
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
