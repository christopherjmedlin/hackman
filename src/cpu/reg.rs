pub struct Registers {
    pub af: u16,
    pub bc: u16,
    pub de: u16,
    pub hl: u16,
}

impl Registers {
    pub fn new() -> Self {
        Registers {
            af: 0,
            bc: 0,
            de: 0,
            hl: 0,
        }
    }

    pub fn a(&mut self) -> u8 {
        self.af as u8
    }

    pub fn write_a(&mut self, byte: u8) {
        self.af = byte as u16;
    }

    pub fn b(&mut self) -> u8 {
        self.bc as u8
    }

    pub fn write_b(&mut self, byte: u8) {
        self.bc = byte as u16;
    }

    pub fn d(&mut self) -> u8 {
        self.de as u8
    }

    pub fn write_d(&mut self, byte: u8) {
        self.de = byte as u16;
    }

    pub fn h(&mut self) -> u8 {
        self.hl as u8
    }

    pub fn write_h(&mut self, byte: u8) {
        self.hl = byte as u16;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_single_byte_registers() {
        let mut reg = Registers::new();
        reg.write_a(0x50);
        reg.bc = 0x1000;
        
        assert_eq!(reg.af, 0x0050);
        assert_eq!(reg.a(), 0x50);
        assert_eq!(reg.b(), 0);
    }
}
