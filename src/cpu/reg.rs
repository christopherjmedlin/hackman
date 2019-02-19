use std::fmt;

pub struct Registers {
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,
    pub l: u8,
    pub a: u8,
    pub f: u8,

    pub ixh: u8,
    pub ixl: u8,
    pub iyh: u8,
    pub iyl: u8,
    pub ix_patched: bool,
    pub iy_patched: bool,
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

            ixh: 0,
            ixl: 0,
            iyh: 0,
            iyl: 0,
            ix_patched: false,
            iy_patched: false,
            sp: 0,
            
            i: 0,
            r: 0,

            pc: 0,
        }
    }
    
    /// Read 8 bit register at index <index> according to the table on
    /// this web page: http://www.z80.info/decoding.htm
    pub fn read_8bit_r(&mut self, index: u8) -> u8 {
        match index {
            0 => self.b,
            1 => self.c,
            2 => self.d,
            3 => self.e,
            4 => {
                if self.ix_patched {
                    self.ixh
                } else if self.iy_patched {
                    self.iyh
                } else {
                    self.h
                }
            }
            5 => {
                if self.ix_patched {
                    self.ixl
                } else if self.iy_patched {
                    self.iyl
                } else {
                    self.l
                }
            }
            6 => self.f,
            7 => self.a,
            _ => 0,
        }
    }

    /// Write byte <byte> to 8 bit register at index <index> according to the
    /// table on the z80 decoding opcodes doc mentioned above
    pub fn write_8bit_r(&mut self, index: u8, byte: u8) {
        match index {
            0 => self.b = byte,
            1 => self.c = byte,
            2 => self.d = byte,
            3 => self.e = byte,
            4 => {
                if self.ix_patched {
                    self.ixh = byte;
                } else if self.iy_patched {
                    self.iyh = byte;
                } else {
                    self.h = byte;
                }
            }
            5 => {
                if self.ix_patched {
                    self.ixl = byte;
                } else if self.iy_patched {
                    self.iyl = byte;
                } else {
                    self.l = byte;
                }
            }
            6 => self.f = byte,
            7 => self.a = byte,
            _ => {}
        }
    }
    
    /// Reads a 16 bit registry according to the tables on the z80
    /// decoding opcodes documentation
    ///
    /// If sp is true, it will use the "rp" table with the stack pointer
    /// as the third index. Otherwise, it will use the rp2 table with the
    /// AF register as the third index.
    pub fn read_16bit_r(&mut self, index: u8, sp: bool) -> u16  {
        match index {
            0 => self.bc(),
            1 => self.de(),
            2 => self.hl(),
            3 => if sp {self.sp} else {self.af()},
            _ => 0
        }
    }
    
    /// Same as read_16bit_r but it instead writes 16 bit integer <word>
    /// to it
    pub fn write_16bit_r(&mut self, index: u8, sp: bool, word: u16) {
        match index {
            0 => self.write_bc(word),
            1 => self.write_de(word),
            2 => self.write_hl(word),
            3 => if sp {self.sp = word} else {self.write_af(word)},
            _ => {}
        }
    }

    pub fn bc(&self) -> u16 {
        (self.b as u16) << 8 | (self.c as u16)
    }

    pub fn de(&self) -> u16 {
        (self.d as u16) << 8 | (self.e as u16)
    }

    pub fn hl(&self) -> u16 {
        if self.ix_patched {
             (self.ixh as u16) << 8 | (self.ixl as u16)
        } else if self.iy_patched {
             (self.iyh as u16) << 8 | (self.iyl as u16)
        } else {
             (self.h as u16) << 8 | (self.l as u16)
        }
    }

    pub fn af(&self) -> u16 {
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
        if self.ix_patched {
            self.ixl = (word & 0xFF) as u8;
            self.ixh = (word >> 8) as u8;
        } else if self.iy_patched {
            self.iyl = (word & 0xFF) as u8;
            self.iyh = (word >> 8) as u8;
        } else {
            self.l = (word & 0xFF) as u8;
            self.h = (word >> 8) as u8;
        }
    }

    pub fn write_af(&mut self, word: u16) {
        self.f = (word & 0xFF) as u8;
        self.a = (word >> 8) as u8;
    }

    pub fn cc(&self, index: usize) -> bool {
        match index {
            0 => {self.f & (1 << 6) == 0},
            1 => {self.f & (1 << 6) != 0},
            2 => {self.f & 1 == 0},
            3 => {self.f & 1 != 0},
            4 => {self.f & (1 << 2) == 0},
            5 => {self.f & (1 << 2) != 0},
            6 => {self.f & (1 << 7) == 0},
            7 => {self.f & (1 << 7) != 0},
            _ => false
        }
    }
    
    /// Set bit <bit> of F register to <set>
    pub fn set_flag(&mut self, bit: usize, set: bool) {
        if set { 
            self.f |= 1 << bit;
        } else {
            self.f &= !(1 << bit);
        }
    }

    pub fn read_flag(&mut self, bit: usize) -> bool {
        self.f & 1 << bit != 0
    }

    pub fn patch_ix(&mut self, patch: bool) {
        self.ix_patched = patch;
    }

    pub fn patch_iy(&mut self, patch: bool) {
        self.iy_patched = patch;
    }
}

impl fmt::Debug for Registers {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "pc: 0x{:x}\n\
                   a: 0x{:x}, f: 0x{:x}, b: 0x{:x}, c: 0x{:x}\n\
                   d: 0x{:x}, e: 0x{:x}, h: 0x{:x}, l: 0x{:x}\n\n\
                   af: 0x{:x}, bc: 0x{:x}, de: 0x{:x}, hl: 0x{:x}, sp: {:x}\n\
                   ixh: 0x{:x}, ixl: 0x{:x}, iyh: 0x{:x}, iyl: 0x{:x}\n\
                   i: 0x{:x}, r: {:x}",

               self.pc, 
               self.a, self.f, self.b, self.c, self.d, self.e, self.h, self.l, 
               self.af(), self.bc(), self.de(), self.hl(), self.sp,
               self.ixh, self.ixl, self.iyh, self.iyl, self.i, self.r)
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
    
    #[test]
    fn test_write_16bit_r() {
        let mut reg = Registers::new();

        reg.write_16bit_r(3, true, 0x1234);
        assert_eq!(reg.sp, 0x1234);
        reg.write_16bit_r(3, false, 0x1234);
        assert_eq!(reg.af(), 0x1234);
    }

    #[test]
    fn test_cc() {
        let mut reg = Registers::new();

        reg.set_flag(0, true);
        assert_eq!(reg.cc(2), false);
        assert_eq!(reg.cc(3), true);

        reg.set_flag(0, false);
        assert_eq!(reg.cc(2), true);
        assert_eq!(reg.cc(3), false);
    }
}
