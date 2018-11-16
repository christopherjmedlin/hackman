mod reg;
pub mod mem;

use cpu::reg::Registers;
use cpu::mem::Memory;

pub struct Z80 {
    reg: Registers,
    altreg: Registers,

    halted: bool,
    interrupts_enabled: bool
}

impl Z80 {
    pub fn new() -> Self {
        Z80 {
            reg: Registers::new(),
            altreg: Registers::new(),

            halted: false,
            interrupts_enabled: false,
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
            (0, 0, 7) => {
                self.shift(7, true, false, memory); 

                self.reg.set_flag(1, false);
                self.reg.set_flag(4, false);
                self.inc_pc();
                4
            },
            // RRCA
            (0, 1, 7) => {
                self.shift(7, true, true, memory);

                self.reg.set_flag(1, false);
                self.reg.set_flag(4, false);
                self.inc_pc();
                4
            },
            // RLA
            (0, 2, 7) => {
                self.shift(7, false, false, memory);

                self.reg.set_flag(1, false);
                self.reg.set_flag(4, false);
                self.inc_pc();
                4
            },
            // RRA
            (0, 3, 7) => {
                self.shift(7, false, true, memory);

                self.reg.set_flag(1, false);
                self.reg.set_flag(4, false);
                self.inc_pc();
                4
            },
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
            },
            // alu[y] r[z]
            (2, _, _) => {
                let val = self.r(z, memory);
                self.alu(y, val);
                4
            },
            // RET cc[y]
            (3, _, 0) => {
                if self.reg.cc(y as usize) {
                    self.ret(memory);
                    11
                } else {
                    self.inc_pc();
                    5
                }
            },
            (3, _, 1) => {
                let q = ((y & 1) != 0) as u8;
                let p: u8 = y >> 1;

                match (p, q) {
                    // POP rp2[p]
                    (_, 0) => {
                        let word = self.pop_stack_16(memory);
                        self.reg.write_16bit_r(p, false, word);
                        self.inc_pc();
                        10
                    },
                    // RET
                    (0, 1) => {
                        self.ret(memory);
                        11
                    },
                    // EXX
                    (1, _) => {
                        let mut temp = self.altreg.bc();
                        self.altreg.write_bc(self.reg.bc());
                        self.reg.write_bc(temp);
                        temp = self.altreg.de();
                        self.altreg.write_de(self.reg.de());
                        self.reg.write_de(temp);
                        temp = self.altreg.hl();
                        self.altreg.write_hl(self.reg.hl());
                        self.reg.write_hl(temp);
                        self.inc_pc();
                        4
                    },
                    // JP HL
                    (2, _) => {
                        self.reg.pc = self.reg.hl();
                        4
                    },
                    // LD SP, HL
                    (3, _) => {
                        self.reg.sp = self.reg.hl();
                        self.inc_pc();
                        4
                    },
                    (_, _) => {4}
                }
            },
            // JP cc[y], nn
            (3, _, 2) => {
                if self.reg.cc(y as usize) {
                    self.reg.pc = nn;
                }
                10
            },
            // JP nn
            (3, 0, 3) => {
                self.reg.pc = nn;
                10
            },
            // CB prefix
            (3, 1, 3) => {
                self.inc_pc();
                let op = memory.read_byte(self.reg.pc);
                self.run_cb_opcode(op, memory);
                4
            },
            // OUT (n), A
            (3, 2, 3) => {
                // TODO Implement!!!
                4
            },
            // IN A, (n)
            (3, 3, 3) => {
                // TODO Implement!!!
                4
            },
            // EX (SP), HL
            (3, 4, 3) => {
                let mut temp = self.reg.l;
                self.reg.l = memory.read_byte(self.reg.sp);
                memory.write_byte(temp, self.reg.sp);
                temp = self.reg.h;
                self.reg.h = memory.read_byte(self.reg.sp + 1);
                memory.write_byte(temp, self.reg.sp + 1);
                self.inc_pc();
                19
            },
            // EX DE, HL
            (3, 5, 3) => {
                let de = self.reg.de();
                let hl = self.reg.hl();
                self.reg.write_de(hl);
                self.reg.write_hl(de);
                self.inc_pc();
                4
            },
            // DI
            (3, 6, 3) => {
                self.interrupts_enabled = false;
                self.inc_pc();
                4
            },
            // EI
            (3, 7, 3) => {
                self.interrupts_enabled = true;
                self.inc_pc();
                4
            },
            // CALL cc[y], nn
            (3, _, 4) => {
                if self.reg.cc(y as usize) {
                    self.call(memory, nn);
                    17
                } else {
                    10
                }
            }
            (3, _, 5) => {
                let q = ((y & 1) != 0) as u8;
                let p: u8 = y >> 1;

                match (p, q) {
                    // PUSH rp2[p]
                    (_, 0) => {
                        let word = self.reg.read_16bit_r(p, false);
                        self.push_stack_16(memory, word);
                        self.inc_pc();
                        11
                    },
                    // CALL nn
                    (0, 1) => {
                        self.call(memory, nn);
                        17
                    },
                    // DD prefix
                    (1, _) => {
                        // TODO implement!!!!
                        4
                    },
                    // ED prefix
                    (2, _) => {
                        // TODO implement!!!!
                        4
                    },
                    // FD prefix
                    (3, _) => {
                        // TODO implement!!!!
                        4
                    },
                    (_, _) => {4}
                }
            },
            // alu[y] n
            (3, _, 6) => {
                self.alu(y, n);
                8
            },
            // RST y*8
            (3, _, 7) => {
                self.call(memory, (y*8) as u16);
                11
            },
            (_, _, _) => {4},
        }
    }

    // runs a CB prefixed opcode
    fn run_cb_opcode(&mut self, opcode: u8, memory: &mut Memory) -> usize {
        let x: u8 = opcode >> 6;
        let y: u8 = (opcode & 0b00111000) >> 3;
        let z: u8 = opcode & 0b00000111;
        
        match x {
            // rot[y] r[z]
            0 => {
                self.inc_pc();
                4
            },
            // BIT y, r[z]
            1 => {
                let val = self.r(z, memory);
                self.reg.set_flag(6, (val & 1 << y) != 0);
                self.reg.set_flag(1, false);
                self.reg.set_flag(4, true);
                self.inc_pc();
                8
            },
            // RES y, r[z]
            2 => {
                let val = self.r(z, memory);
                self.write_r(z, val & !(1 << y), memory);
                self.inc_pc();
                8
            },
            // SET y, r[z]
            3 => {
                let val = self.r(z, memory);
                self.write_r(z, val | (1 << y), memory);
                self.inc_pc();
                8 
            },
            _ => {
                4
            }

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

    fn alu(&mut self, operator: u8, val: u8) {
        match operator {
            // ADD A,
            0 => {self.add(val, false)},
            // ADC a,
            1 => {self.add(val, true)},
            // SUB a,
            2 => {self.sub(val, false)},
            // SBC a,
            3 => {self.sub(val, true)},
            // AND a,
            4 => {self.and(val)},
            // XOR a,
            5 => {self.xor(val)},
            // OR a,
            6 => {self.or(val)},
            // CP a,
            7 => {self.cp(val)},
            _ => {}
        }
        self.inc_pc();
    }

    fn add(&mut self, val: u8, add_carry: bool) {
        let left = self.reg.a;
        let right = val;
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

    fn sub(&mut self, val: u8, sub_carry: bool) {
        let left = self.reg.a;
        let right = val;
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

    fn and(&mut self, val: u8) {
        let result = self.reg.a & val;
        self.reg.a = result;

        self.detect_parity(result);
        self.reg.set_flag(0, false);
        self.reg.set_flag(1, false);
        self.reg.set_flag(4, true);
        self.reg.set_flag(6, result == 0);
        self.reg.set_flag(7, result > 127);
    }

    fn xor(&mut self, val: u8) {
        let result = self.reg.a ^ val;
        self.reg.a = result;

        self.detect_parity(result);
        self.reg.set_flag(0, false);
        self.reg.set_flag(1, false);
        self.reg.set_flag(4, false);
        self.reg.set_flag(6, result == 0);
        self.reg.set_flag(7, result > 127);
    }

    fn or(&mut self, val: u8) {
        let result = self.reg.a | val;
        self.reg.a = result;

        self.detect_parity(result);
        self.reg.set_flag(0, false);
        self.reg.set_flag(1, false);
        self.reg.set_flag(4, false);
        self.reg.set_flag(6, result == 0);
        self.reg.set_flag(7, result > 127);
    }

    fn cp(&mut self, val: u8) {
        let left = self.reg.a;
        let right = val;
        let mut result = left.wrapping_sub(right);

        self.detect_overflow_sub(right, left, false);
        self.detect_half_carry_sub(right, left, false);

        self.reg.set_flag(0, (left + right) as u16 > 255);
        self.reg.set_flag(1, true);
        self.reg.set_flag(6, result == 0);
        self.reg.set_flag(7, result > 127);
    }

    fn rot(&mut self, operator: u8, val: u8, mem: &mut Memory) {
        match operator {
            // RLC
            0 => {self.shift(val, true, false, mem);},
            // RRC
            1 => {self.shift(val, true, true, mem);},
            // RL
            2 => {self.shift(val, false, false, mem);},
            // RR
            3 => {self.shift(val, false, true, mem);},
            // SLA
            _ => {},
        }

        let val = self.r(val, mem);
        self.reg.set_flag(1, false);
        self.reg.set_flag(4, false);
        self.reg.set_flag(6, val == 0);
        self.reg.set_flag(7, (val as i8) >= 0);
        self.detect_parity(val);
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
    fn shift(&mut self, register: u8, carry_bit: bool, 
                 right: bool, mem: &mut Memory) {
        let mut value = self.r(register, mem);

        if right {
            value >>= 1;
        } else {
            value <<= 1;
        }
        let carry_mask = if right {0b00000001} else {0b10000000};
        let carry = (value & carry_mask) != 0;
        
        let mut mask_shift = 0;
        // if it is a right shift, the 0th bit should be carried to
        // the 7th bit, not the other way around
        if right {
            mask_shift = 7;
        }

        let bit = if carry_bit {carry} else {self.reg.cc(3)};
        if bit {
            value |= 1 << mask_shift;
        } else {
            value &= !(1 << mask_shift);
        }
        
        self.write_r(register, value, mem);
        self.reg.set_flag(0, carry);
    }

    // shift but a zero is copied to bit

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
    
    // pushes pc + 3 to stack and then jumps to address <addr>
    fn call(&mut self, mem: &mut Memory, addr: u16) {
        let ret_addr = self.reg.pc + 3;
        self.push_stack_16(mem, ret_addr);
        self.reg.pc = addr;
    }

    // pops top stack entry into pc
    fn ret(&mut self, mem: &mut Memory) {
        self.reg.pc = self.pop_stack_16(mem);      
    }
    
    // returns byte at memory address pointed to by stack pointer and then
    // increments stack pointer
    fn pop_stack(&mut self, mem: &mut Memory) -> u8 {
        let byte = mem.read_byte(self.reg.sp);
        self.reg.sp += 1;
        byte
    }

    // saves byte at memory address pointed to by stack pointer and then
    // decrements stack pointer
    fn push_stack(&mut self, mem: &mut Memory, byte: u8) {
        if self.reg.sp > 0 {
            self.reg.sp -= 1;
        } else {
            println!("Stack overflow!");
        }
        mem.write_byte(byte, self.reg.sp);
    }

    fn push_stack_16(&mut self, mem: &mut Memory, word: u16) {
        self.push_stack(mem, ((word & 0xFF00) >> 8) as u8);
        self.push_stack(mem, (word & 0x00FF) as u8);
    }

    fn pop_stack_16(&mut self, mem: &mut Memory) -> u16 {
        let mut word = self.pop_stack(mem) as u16;
        word |= (self.pop_stack(mem) as u16) << 8;
        return word;
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
