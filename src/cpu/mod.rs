mod reg;
pub mod mem;

use cpu::reg::Registers;
use cpu::mem::Memory;

pub struct Z80 {
    reg: Registers,
    altreg: Registers,

    halted: bool,
}

impl Z80 {
    pub fn new() -> Self {
        Z80 {
            reg: Registers::new(),
            altreg: Registers::new(),

            halted: false,
        }
    }
    
    /// Runs a specified number of opcodes
    pub fn run_opcodes(&mut self, iters: usize, memory: &mut Memory) -> usize {
        if self.halted {
            return 0;
        }
        
        let mut cycles = 0;
        for i in 0..iters {
            let opcode = memory.read_byte(self.reg.pc);
            cycles += self.run_opcode(opcode, memory);
        }

        cycles
    }

    fn run_opcode(&mut self, opcode: u8, memory: &mut Memory) -> usize {
        let n: u8 = memory.read_byte(self.reg.pc + 1);
        let nn: u16 = (memory.read_byte(self.reg.pc + 1) as u16) << 8 | 
                      (memory.read_byte(self.reg.pc + 2) as u16);
        let d: i8 = n as i8;

        let x: u8 = opcode >> 6;
        let y: u8 = (opcode & 0b00111000) >> 3;
        let z: u8 = opcode & 0b00000111;

        match (x, y, z) {
            // NOP
            (0, 0, 0) => {self.inc_pc(); 4},
            // EX AF, AF'
            (0, 1, 0) => {
                let temp = self.altreg.af();
                self.altreg.write_af(self.reg.af());
                self.reg.write_af(temp);
                4
            },
            // DJNZ d
            (0, 2, 0) => {
                self.reg.b -= 1;
                if self.reg.b > 0 {
                    self.jr(d);
                    13
                } else {
                    8
                }
            },
            // JR d
            (0, 3, 0) => {
                self.jr(d);
                12
            },
            // JR cc[y-4], d
            (0, 4...7, 0) => {
                if self.reg.cc((y - 4) as usize) {
                    self.jr(d);
                    return 12;
                }
                7
            },
            (0, _, 1) => {
                let q = (y & 1) != 0;
                let p: u8 = y >> 1;
                
                // ADD HL, rp[p]
                if q {

                    let result = self.reg.hl().wrapping_add(
                        self.reg.read_16bit_r(p, true)
                    );
                    self.reg.write_hl(result);
                    11
                }
                // LD rp[p], nn
                else {
                    self.reg.write_16bit_r(p, true, nn);
                    10
                }
            }
            (0, _, 2) => {
                let q = (y & 1) != 0;
                let p: u8 = y >> 1;
                
                if q {
                    match p {
                        // LD A, (BC)
                        0 => {
                            self.reg.a = memory.read_byte(self.reg.bc());
                            self.inc_pc();
                            7
                        },
                        // LD A, (DE)
                        1 => {
                            self.reg.a = memory.read_byte(self.reg.de());
                            self.inc_pc();
                            7
                        },
                        // LD HL, (nn)
                        2 => {
                            self.reg.write_hl(memory.read_word(nn));
                            self.reg.pc += 3;
                            16
                        }
                        // LD A, (nn)
                        3 => {
                            self.reg.a = memory.read_byte(nn);
                            self.reg.pc += 3;
                            13
                        },
                        _ => {self.inc_pc(); 4}
                    }
                } else {
                    match p {
                        // LD (BC), A
                        0 => {
                            memory.write_byte(self.reg.a, self.reg.bc());
                            self.inc_pc();
                            7
                        },
                        // LD (DE), A
                        1 => {
                            memory.write_byte(self.reg.a, self.reg.de());
                            self.inc_pc();
                            7
                        },
                        // LD (nn), HL
                        2 => {
                            memory.write_word(self.reg.hl(), nn);
                            self.reg.pc += 3;
                            16
                        },
                        // LD (nn), A
                        3 => {
                            memory.write_byte(self.reg.a, nn);
                            self.reg.pc += 3;
                            13
                        },
                        _ => {self.inc_pc(); 4}
                    }
                }
            },
            (0, _, 3) => {
                let q = (y & 1) != 0;
                let p: u8 = y >> 1;
                
                // DEC rp[p]
                if q {self.dec_16(p); 6}
                // INC rp[p]
                else {self.inc_16(p); 6}
            },
            // INC r[y]
            (0, _, 4) => {self.inc_8(y, memory); 4},
            // DEC r[y]
            (0, _, 5) => {self.dec_8(y, memory); 4},
            // LD r[y], n
            (0, _, 6) => {
                self.write_r(y, n, memory);
                self.inc_pc();
                7
            },
            // RLCA
            (0, 0, 7) => {self.acc_shift(true, false); 4},
            // RRCA
            (0, 1, 7) => {self.acc_shift(true, true); 4},
            // RLA
            (0, 2, 7) => {self.acc_shift(false, false); 4},
            // RRA
            (0, 3, 7) => {self.acc_shift(false, true); 4},
            // DAA
            (0, 4, 7) => {
                if self.reg.a & 0xF0 > 9 || self.reg.read_flag(4) {
                    self.reg.a += 6;
                }
                if (self.reg.a & 0xF0) >> 4 > 9 {
                    self.reg.a += 0x60;
                    self.reg.set_flag(0, true);
                }
                let num = self.reg.a;
                self.detect_parity(num);
                self.inc_pc();
                4
            },
            // CPL
            (0, 5, 7) => {
                self.reg.a = !self.reg.a;
                self.inc_pc();
                // set H and N
                self.reg.set_flag(1, true);
                self.reg.set_flag(4, true);
                4
            },
            // SCF
            (0, 6, 7) => {
                self.reg.set_flag(0, true);
                self.inc_pc();
                // reset H and N
                self.reg.set_flag(1, false);
                self.reg.set_flag(4, false);
                4
            },
            // CCF
            (0, 7, 7) => {
                // set H to old C
                let old_c = self.reg.read_flag(0);
                self.reg.set_flag(4, old_c);
                // reset N
                self.reg.set_flag(1, false);
                // inverse C
                let inverse_c = !self.reg.read_flag(0);
                self.reg.set_flag(0, inverse_c);
                4
            },
            // HALT
            (1, 6, 6) => {
                self.halted = true;
                4
            },
            // LD r[y], r[z]
            (1, _, _) => {
                let temp = self.r(z, memory);
                self.write_r(y, temp, memory);
                4
            }
            // alu[y] r[z]
            (2, _, _) => {
                self.alu(y, z, memory);
                4
            }

            (_, _, _) => {4},
        }
    }
    
    // implements the r table in the decoding opcodes documentation with (hl)
    // at 6
    fn r(&mut self, index: u8, mem: &mut Memory) -> u8 {
        if index == 6 {
            mem.read_byte(self.reg.hl())
        } else {
            self.reg.read_8bit_r(index)
        }
    }

    fn write_r(&mut self, index: u8, byte: u8, mem: &mut Memory) {
        if index == 6 {
            mem.write_byte(byte, self.reg.hl());
        } else {
            self.reg.write_8bit_r(index, byte);
        }
    }

    fn alu(&mut self, y: u8, z: u8, mem: &mut Memory) {
        match y {
            // ADD A,
            0 => {self.add(z, false, mem)},
            // ADC a,
            1 => {self.add(z, true, mem)},
            // SUB a,
            2 => {self.sub(z, false, mem)},
            // SBC a,
            3 => {self.sub(z, true, mem)},
            // AND a,
            4 => {},
            // XOR a,
            5 => {},
            // OR a,
            6 => {},
            // CP a,
            7 => {},
            _ => {}
        }
        self.inc_pc();
    }

    fn add(&mut self, z: u8, add_carry: bool, mem: &mut Memory) {
        let left = self.reg.a;
        let right = self.r(z, mem);
        let mut result = right.wrapping_add(left);

        if add_carry {
            let carry = self.reg.read_flag(0);
            result = result.wrapping_add(carry as u8);
            self.detect_overflow_add(right, left, carry);
            self.detect_half_carry_add(right, left, carry);
        } else {
            self.detect_overflow_add(right, left, false);
            self.detect_half_carry_add(right, left, false);
        }
        self.reg.a = result;
    
        self.reg.set_flag(0, (left + right) as u16 > 255);
        self.reg.set_flag(1, false);
        self.reg.set_flag(6, result == 0);
        self.reg.set_flag(7, result > 127);
    }

    fn sub(&mut self, z: u8, sub_carry: bool, mem: &mut Memory) {
        let left = self.reg.a;
        let right = self.r(z, mem);
        let mut result = left.wrapping_sub(right);

        if sub_carry {
            let carry = self.reg.read_flag(0);
            result = result.wrapping_sub(carry as u8);
            self.detect_overflow_sub(right, left, carry);
            self.detect_half_carry_sub(right, left, carry);
        } else {
            self.detect_overflow_sub(right, left, false);
            self.detect_half_carry_sub(right, left, false);
        }
        self.reg.a = result;

        self.reg.set_flag(0, (left + right) as u16 > 255);
        self.reg.set_flag(1, true);
        self.reg.set_flag(6, result == 0);
        self.reg.set_flag(7, result > 127);
    }

    fn and(&mut self, z: u8, mem: &mut Memory) {
        let result = self.reg.a & self.r(z, mem);
        self.reg.a = result;

        self.detect_parity(result);
        self.reg.set_flag(0, false);
        self.reg.set_flag(1, false);
        self.reg.set_flag(4, true);
        self.reg.set_flag(6, result == 0);
        self.reg.set_flag(7, result > 127);
    }

    fn inc_pc(&mut self) {
        self.reg.pc += 1;
    }
    
    // adds d to pc
    fn jr(&mut self, d: i8) {
        let result = self.reg.pc as i8 + d;
        if result < 0 {
            self.reg.pc = 0;
        } else {
            self.reg.pc = result as u16;
        }
    }
    
    // used for simplifying RLCA, RLA, RRCA, RRA instructions
    //
    // if carry_bit is true, the 7th bit is carried over to the 0th,
    // otherwise the 0th is set to the carry flag before the instruction
    //
    // if right is true, a right shift is performed. left otherwise
    fn acc_shift(&mut self, carry_bit: bool, right: bool) {
        if right {
            self.reg.a >>= 1;
        } else {
            self.reg.a <<= 1;
        }
        let carry_mask = if right {0b00000001} else {0b10000000};
        let carry = (self.reg.a & carry_mask) != 0;
        
        let mut mask_shift = 0;
        // if it is a right shift, the 0th bit should be carried to
        // the 7th bit, not the other way around
        if right {
            mask_shift = 7;
        }

        let bit = if carry_bit {carry} else {self.reg.cc(3)};
        if bit {
            self.reg.a |= 1 << mask_shift;
        } else {
            self.reg.a &= !(1 << mask_shift);
        }

        self.reg.set_flag(0, carry);
        self.reg.set_flag(1, false);
        self.reg.set_flag(4, false);
        self.inc_pc();
    }

    // decrements register at p and increments pc
    fn dec_16(&mut self, p: u8) {
        let result = self.reg.read_16bit_r(p, true) - 1;
        self.reg.write_16bit_r(p, true, result);
        self.inc_pc();
    }
    
    // increments register at p and increments pc
    fn inc_16(&mut self, p: u8) {
        let result = self.reg.read_16bit_r(p, true) + 1;
        self.reg.write_16bit_r(p, true, result);
        self.inc_pc();
    }
    
    // decrements register at y and increments pc
    fn dec_8(&mut self, y: u8, mem: &mut Memory) {
        let val = self.reg.read_8bit_r(y);
        let result = val.wrapping_sub(1);

        self.reg.set_flag(1, true);
        self.reg.set_flag(6, result == 0);
        self.reg.set_flag(7, result > 127);
        self.detect_half_carry_add(val, 1, false);
        self.detect_overflow_add(val, 1, false);
        self.write_r(y, result, mem);
        self.inc_pc();
    }

    // icrements register at y and increments pc
    fn inc_8(&mut self, y: u8, mem: &mut Memory) {
        let val = self.reg.read_8bit_r(y);
        let result = val.wrapping_add(1);
        
        self.reg.set_flag(1, false);
        self.reg.set_flag(6, result == 0);
        self.reg.set_flag(7, result > 127);
        self.detect_half_carry_sub(val, 1, false);
        self.detect_overflow_sub(val, 1, false);

        self.write_r(y, result, mem);
        self.inc_pc();
    }

    // detects if a half carry occurs in an operation left + right
    // and sets flag accordingly
    fn detect_half_carry_add(&mut self, left: u8, right: u8, carry: bool) {
        let mut left = left + (carry as u8);
        self.reg.set_flag(4, (left & 0x0F) + (right & 0x0F) > 0x0F);
    }

    // detects if a half carry occurs in operation left - right
    // and sets flag accordingly
    fn detect_half_carry_sub(&mut self, left: u8, right: u8, carry: bool) {
        let mut left = left - (carry as u8);
        self.reg.set_flag(4, (left & 0x0F).wrapping_sub(right & 0x0F) > 0x0F);
    }

    // detects if an overflow occurs in operation left + right
    // and sets flag accordingly
    fn detect_overflow_add(&mut self, left: u8, right: u8, carry: bool) {
        let mut left = left as i8 as i32;
        let right = right as i8 as i32;
        
        left = left + right + (carry as i32);
        self.reg.set_flag(2, (left < -128) || (left > 127));
    }

    // detects if an overflow occurs in operation left - right
    // and sets flag accordingly
    fn detect_overflow_sub(&mut self, left: u8, right: u8, carry: bool) {
        let mut left = left as i8 as i32;
        let right = right as i8 as i32;
 
        left = left - right - (carry as i32);
        self.reg.set_flag(2, (left < -128) || (left > 127));
    }
        
    // sets parity flag accordingly to the given number
    fn detect_parity(&mut self, mut num: u8) {
        let mut number_of_ones = 0;
        while num != 0 {
            if num & 1 != 0 {number_of_ones += 1};
            num >>= 1;
        }
    }
}

#[cfg(test)]
mod tests;
