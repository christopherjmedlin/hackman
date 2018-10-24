mod reg;
pub mod mem;

use cpu::reg::Registers;
use cpu::mem::Memory;

pub struct Z80 {
    reg: Registers,
    altreg: Registers,
}

impl Z80 {
    pub fn new() -> Self {
        Z80 {
            reg: Registers::new(),
            altreg: Registers::new(),
        }
    }
    
    /// Runs a specified number of opcodes
    pub fn run_opcodes(&mut self, iters: usize, memory: &mut Memory) {
        for i in 0..iters {
            let opcode = memory.read_byte(self.reg.pc);
            self.run_opcode(opcode, memory);
            self.reg.pc += 1;
        }
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
            (0, 0, 0) => {4},
            // EX AF, AF'
            (0, 1, 0) => {
                //TODO needs to be implemented!!!
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
                        0 => {self.reg.a = memory.read_byte(self.reg.bc()); 7},
                        // LD A, (DE)
                        1 => {self.reg.a = memory.read_byte(self.reg.de()); 7},
                        // LD HL, (nn)
                        2 => {self.reg.write_hl(memory.read_word(nn)); 16}
                        // LD A, (nn)
                        3 => {self.reg.a = memory.read_byte(nn); 13},
                        _ => {4}
                    }
                } else {
                    match p {
                        // LD (BC), A
                        0 => {memory.write_byte(self.reg.a, self.reg.bc()); 7},
                        // LD (DE), A
                        1 => {memory.write_byte(self.reg.a, self.reg.de()); 7},
                        // LD (nn), HL
                        2 => {memory.write_word(self.reg.hl(), nn); 16},
                        // LD (nn), A
                        3 => {memory.write_byte(self.reg.a, nn); 13},
                        _ => {4}
                    }
                }
            },
            (0, _, 3) => {
                let q = (y & 1) != 0;
                let p: u8 = y >> 1;
                
                // DEC rp[p]
                if q {
                    self.dec(p);
                    6
                }
                // INC rp[p]
                else {
                    self.inc(p);
                    6
                }
            },

            
            (_, _, _) => {4},
        }
    }

    fn jr(&mut self, d: i8) {
        let result = self.reg.pc as i8 + d;
        if result < 0 {
            self.reg.pc = 0;
        } else {
            self.reg.pc = result as u16;
        }
        self.reg.pc += 1;
    }

    fn dec(&mut self, p: u8) {
        let result = self.reg.read_16bit_r(p, true) - 1;
        self.reg.write_16bit_r(
            p, true, result
        );
    }

    fn inc(&mut self, p: u8) {
        let result = self.reg.read_16bit_r(p, true) + 1;
        self.reg.write_16bit_r(
            p, true, result
        );
    }
}

#[cfg(test)]
mod tests;
