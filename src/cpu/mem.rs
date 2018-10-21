pub trait Memory {
    
    /// Read a byte from memory at address <addr>
    pub fn read_byte(&self, addr: u16) -> u8;

    /// Write byte <byte> to memory at address <addr>
    pub fn write_byte(&mut self, byte: u8, addr: u16);
}
