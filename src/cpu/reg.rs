pub struct Registers {
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,
    pub l: u8,
    pub a: u8,
    pub f: u8,

    pub ix: u16,
    pub iy: u16,
    pub sp: u16,

    pub i: u8,
    pub r: u8,

    pub pc: u16,
}

impl Registers {
    pub fn new() -> Self {
        Registers {
            a: 0,
            f: 0,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            h: 0,
            l: 0,

            ix: 0,
            iy: 0,
            sp: 0,
            
            i: 0,
            r: 0,

            pc: 0,
        }
    }
    
    /// Read 8 bit register at index <index> according to the table on
    /// this web page: http://www.z80.info/decoding.htm
    pub fn read_8bit_r(&mut self, index: usize) -> u8 {
        match index {
            0 => self.b,
            1 => self.c,
            2 => self.d,
            3 => self.e,
            4 => self.h,
            5 => self.l,
            6 => self.f,
            7 => self.a,
            _ => 0,
        }
    }

    /// Write byte <byte> to 8 bit register at index <index> according to the
    /// table on the z80 decoding opcodes doc mentioned above
    pub fn write_8bit_r(&mut self, index: usize, byte: u8) {
        match index {
            0 => self.b = byte,
            1 => self.c = byte,
            2 => self.d = byte,
            3 => self.e = byte,
            4 => self.h = byte,
            5 => self.l = byte,
            6 => self.f = byte,
            7 => self.a = byte,
            _ => {}
        }
    }

    pub fn bc(&mut self) -> u16 {
        (self.b as u16) << 8 | (self.c as u16)
    }

    pub fn de(&mut self) -> u16 {
        (self.d as u16) << 8 | (self.e as u16)
    }

    pub fn hl(&mut self) -> u16 {
        (self.h as u16) << 8 | (self.l as u16)
    }

    pub fn af(&mut self) -> u16 {
        (self.a as u16) << 8 | (self.f as u16)
    }

    pub fn write_bc(&mut self, word: u16) {
        self.c = (word & 0xFF) as u8;
        self.b = (word >> 8) as u8;
    }

    pub fn write_de(&mut self, word: u16) {
        self.e = (word & 0xFF) as u8;
        self.d = (word >> 8) as u8;
    }

    pub fn write_hl(&mut self, word: u16) {
        self.l = (word & 0xFF) as u8;
        self.h = (word >> 8) as u8;
    }

    pub fn write_af(&mut self, word: u16) {
        self.f = (word & 0xFF) as u8;
        self.a = (word >> 8) as u8;
    }

    pub fn cc(&self, index: usize) {
        match index {
            
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_16bit_registers() {
        let mut reg = Registers::new();

        reg.write_bc(0x1234);
        assert_eq!(reg.bc(), 0x1234);
        assert_eq!(reg.b, 0x12);
        assert_eq!(reg.c, 0x34);

        reg.write_de(0x1234);
        assert_eq!(reg.de(), 0x1234);
        assert_eq!(reg.d, 0x12);
        assert_eq!(reg.e, 0x34);

        reg.write_hl(0x1234);
        assert_eq!(reg.hl(), 0x1234);
        assert_eq!(reg.h, 0x12);
        assert_eq!(reg.l, 0x34);

        reg.write_af(0x1234);
        assert_eq!(reg.af(), 0x1234);
        assert_eq!(reg.a, 0x12);
        assert_eq!(reg.f, 0x34);
    }
}
