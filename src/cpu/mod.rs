mod reg;
pub mod mem;
pub mod io;

use std::fmt;

use cpu::reg::Registers;
use cpu::mem::Memory;
use cpu::io::InputOutput;

// TODO make IO into a trait
pub struct Z80 {
    reg: Registers,
    altreg: Registers,

    halted: bool,
    interrupts_enabled: bool,
    interrupt: bool,
    interrupt_data: u8,
    interrupt_mode: u8,
}

impl Z80 {
    pub fn new() -> Self {
        let mut cpu = Z80 {
            reg: Registers::new(),
            altreg: Registers::new(),

            halted: false,
            interrupts_enabled: true,
            interrupt: false,
            interrupt_data: 0,
            interrupt_mode: 0
        };
        cpu.reg.sp = 0x4FEF;
        cpu
    }

    pub fn interrupt(&mut self, data: u8) {
        self.halted = false;
        self.interrupt = true;
        self.interrupt_data = data;
    }
    
    /// Runs a specified number of opcodes
    pub fn run_opcodes(&mut self, iters: usize, memory: &mut Memory, io: &mut InputOutput) -> usize {
        if self.halted {
            return 4;
        }
        
        let mut cycles = 0;
        for i in 0..iters {
            let opcode = memory.read_byte(self.reg.pc);
            cycles += self.run_opcode(opcode, memory, io, false);
        }

        cycles
    }
    
    // mostly for debugging purposes
    pub fn get_pc(&mut self) -> u16 {
        return self.reg.pc;
    }

    fn run_opcode(&mut self, opcode: u8, memory: &mut Memory, io: &mut InputOutput, ext: bool) -> usize {       
        if self.interrupts_enabled {
            if self.interrupt {
                match self.interrupt_mode {
                    _ => {
                        self.interrupt = false;
                        let addr = memory.read_word(((self.reg.i as u16) << 8) | self.interrupt_data as u16);
                        println!("{:x}", ((self.reg.i as u16) << 8) | self.interrupt_data as u16);
                        println!("sdf{:x}", addr);

                        // push pc onto stack
                        let pc = self.reg.pc;
                        self.push_stack_16(memory, pc);
                        self.reg.pc = addr;
                    }
                }
            }
        }
        
        let n: u8 = memory.read_byte(self.reg.pc + 1);
        let nn: u16 = memory.read_word(self.reg.pc + 1);
        println!("{:x}, {:x}", self.reg.pc, nn);
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
                if self.reg.b > 0 {self.reg.b -= 1;}
                if self.reg.b > 0 {
                    self.jr(d);
                    13
                } else {
                    self.reg.pc += 2;
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
                else {self.reg.pc += 2;}
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
                    self.inc_pc();
                    11
                }
                // LD rp[p], nn
                else {
                    println!("{:x}, {:x}", self.reg.pc, nn);
                    self.reg.write_16bit_r(p, true, nn);
                    self.reg.pc += 3;
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
                self.reg.pc += 2;
                7
            },
            // RLCA
            (0, 0, 7) => {
                let val = self.reg.a;
                self.reg.a = self.rot(0, val);
                self.inc_pc();
                4
            },
            // RRCA
            (0, 1, 7) => {
                let val = self.reg.a;
                self.reg.a = self.rot(1, val);
                self.inc_pc();
                4
            },
            // RLA
            (0, 2, 7) => {
                let val = self.reg.a;
                self.reg.a = self.rot(2, val);
                self.inc_pc();
                4
            },
            // RRA
            (0, 3, 7) => {
                let val = self.reg.a;
                self.reg.a = self.rot(3, val);
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
                self.inc_pc();
                4
            },
            // HALT
            (1, 6, 6) => {
                self.halted = true;
                self.inc_pc();
                4
            },
            // LD r[y], r[z]
            (1, _, _) => {
                let temp = self.r(z, memory);
                self.write_r(y, temp, memory);
                self.inc_pc();
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
                self.run_cb_opcode(memory, io, ext);
                4
            },
            // OUT (n), A
            (3, 2, 3) => {
                io.output(n, self.reg.a);
                self.inc_pc();
                4
            },
            // IN A, (n)
            (3, 3, 3) => {
                self.reg.a = io.input(n);
                self.inc_pc();
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
                        self.inc_pc();
                        let opcode = memory.read_byte(self.reg.pc);
                        self.reg.patch_ix(true);
                        let cycles = self.run_opcode(opcode, memory, io, true);
                        self.reg.patch_ix(false);
                        cycles
                    },
                    // ED prefix
                    (2, _) => {
                        self.inc_pc();
                        let op = memory.read_byte(self.reg.pc);
                        self.run_ed_opcode(op, memory, io);
                        4
                    },
                    // FD prefix
                    (3, _) => {
                        self.inc_pc();
                        let opcode = memory.read_byte(self.reg.pc);
                        self.reg.patch_iy(true);
                        let cycles = self.run_opcode(opcode, memory, io, true);
                        self.reg.patch_iy(false);
                        cycles
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
    fn run_cb_opcode(&mut self, memory: &mut Memory, io: &mut InputOutput, ext: bool) -> usize {
        let mut d: u16 = 0;
        if ext {
            // cast to u16 because addresses are 16 bit
            d = memory.read_byte(self.reg.pc) as u16;
            // account for displacement byte
            self.inc_pc();
        }

        let opcode = memory.read_byte(self.reg.pc);
        let x: u8 = opcode >> 6;
        let y: u8 = (opcode & 0b00111000) >> 3;
        let z: u8 = opcode & 0b00000111;

        match x {
            // rot[y] r[z]
            0 => {
                if z == 6 {
                    let val = memory.read_byte(self.reg.hl() + d);
                    let result = self.rot(y, val);
                    memory.write_byte(result, self.reg.hl() + d);
                } else if ext {
                    let val = memory.read_byte(self.reg.hl() + d);
                    let result = self.rot(y, val);
                    memory.write_byte(result, self.reg.hl() + d);
                    self.write_r(z, result, memory);
                } else {
                    let val = self.r(z, memory);
                    let result = self.rot(y, val);
                    self.write_r(z, result, memory);
                }
                self.inc_pc();
                4
            },
            // BIT y, r[z]
            1 => {
                // (IX + d) if ext, r[z] otherwise
                let val = if ext {
                    memory.read_byte(self.reg.hl() + d)
                } else {
                    self.r(z, memory)
                };
                self.reg.set_flag(6, (val & 1 << y) != 0);
                self.reg.set_flag(1, false);
                self.reg.set_flag(4, true);
                self.inc_pc();
                8
            },
            // RES y, r[z]
            2 => {
                if z == 6 {
                    let val = memory.read_byte(self.reg.hl() + d);
                    memory.write_byte(val & !(1 << y), self.reg.hl() + d);
                } else if ext {
                    let val = memory.read_byte(self.reg.hl() + d);
                    let result = val & !(1 << y);
                    memory.write_byte(result, self.reg.hl() + d);
                    self.write_r(z, result, memory);
                } else {
                    let val = self.r(z, memory);
                    self.write_r(z, val & !(1 << y), memory);
                }
                self.inc_pc();
                8
            },
            // SET y, r[z]
            3 => {
                if z == 6 {
                    let val = memory.read_byte(self.reg.hl() + d);
                    memory.write_byte(val | (1 << y), self.reg.hl() + d);
                } else if ext {
                    let val = memory.read_byte(self.reg.hl() + d);
                    let result = val | (1 << y);
                    memory.write_byte(result, self.reg.hl() + d);
                    self.write_r(z, result, memory);
                } else {
                    let val = self.r(z, memory);
                    self.write_r(z, val | (1 << y), memory);
                }
                self.inc_pc();
                8 
            },
            _ => {
                4
            }
        }
    }

    // runs an ED prefixed opcode
    fn run_ed_opcode(&mut self, opcode: u8, memory: &mut Memory, io: &mut InputOutput) -> usize {
        let nn: u16 = memory.read_word(self.reg.pc + 1);

        let x: u8 = opcode >> 6;
        let y: u8 = (opcode & 0b00111000) >> 3;
        let z: u8 = opcode & 0b00000111;

        match (x, y, z) {
            // IN (C)
            (1, 6, 0) => {
                let val = io.input(self.reg.c);

                self.detect_parity(val);
                self.reg.set_flag(1, false);
                self.reg.set_flag(4, false);
                self.reg.set_flag(6, val == 0);
                self.reg.set_flag(7, val > 127);
                self.inc_pc();
                12
            },
            // IN r[y], (C)
            (1, _, 0) => {
                let val = io.input(self.reg.c);

                self.detect_parity(val);
                self.reg.set_flag(1, false);
                self.reg.set_flag(4, false);
                self.reg.set_flag(6, val == 0);
                self.reg.set_flag(7, val > 127);
                self.inc_pc();
                self.reg.write_8bit_r(y, val);
                12
            },
            // OUT (C), 0
            (1, 6, 1) => {
                io.output(self.reg.c, 0);
                self.inc_pc();
                12
            },
            // OUT (C), r[y]
            (1, _, 1) => {
                let val = self.reg.read_8bit_r(y);
                io.output(self.reg.c, val);
                self.inc_pc();
                12
            },
            (1, _, 2) => {
                let q = (y & 1) != 0;
                let p: u8 = y >> 1;
                let value = self.reg.read_16bit_r(p, true);

                // ADC HL, rp[p]
                if q {
                    self.add_16(value, true);
                    self.inc_pc();
                    15
                }
                // SBC HL, rp[p]
                else {
                    self.sub_16(value, true);
                    self.inc_pc();
                    15
                }
            },
            (1, _, 3) => {
                let q = (y & 1) != 0;
                let p: u8 = y >> 1;

                // LD rp[p], (nn)
                if q {
                    let val = memory.read_word(nn);
                    self.reg.write_16bit_r(p, true, val);
                    self.inc_pc();
                    20
                }
                // LD (nn), rp[p]
                else {
                    let val = self.reg.read_16bit_r(p, true);
                    memory.write_word(val, nn);
                    self.inc_pc();
                    20
                }
            },
            // NEG
            (1, _, 4) => {
                let neg: i8 = 0;
                let a = self.reg.a;
                self.detect_overflow_sub(0, a, false);
                self.detect_half_carry_sub(0, a, false);
                let val = neg.wrapping_sub(a as i8) as u8;
                self.reg.a = val;
                self.reg.set_flag(0, false);
                self.reg.set_flag(1, true);
                self.reg.set_flag(6, val == 0);
                self.reg.set_flag(7, val > 127);
                8
            },
            // TODO i think these are supposed to do something other than just returning
            // RETI
            (1, 1, 5) => {
                self.ret(memory);
                14
            },
            // RETN
            (1, _, 5) => {
                self.ret(memory);
                14
            },
            // IM
            (1, _, 6) => {
                self.interrupt_mode = match y {
                    0 | 4 => 0,
                    1 | 5 | 2 | 6 => 1,
                    _ => 2
                };
                self.inc_pc();
                8
            },
            // LD I, A
            (1, 0, 7) => {
                self.reg.i = self.reg.a;
                self.inc_pc();
                println!("{:x}", self.reg.i);
                9
            },
            // LD R, A
            (1, 1, 7) => {
                self.reg.r = self.reg.a;
                self.inc_pc();
                9
            },
            // LD A, I
            (1, 2, 7) => {
                self.reg.a = self.reg.i;
                self.inc_pc();
                9
            },
            // LD A, R
            (1, 3, 7) => {
                self.reg.a = self.reg.r;
                self.inc_pc();
                9
            },
            // RRD
            (1, 4, 7) => {
                // i have no clue if this is even used in pacman so f it
                // TODO maybe implement in the future if i plan to reuse this z80 code
                println!("RRD");
                18
            },
            // RLD
            (1, 5, 7) => {
                // see above
                println!("RLD");
                18
            },
            (2, _, 0...3) => {
                self.bli(y, z, memory, io);
                16
            },
            (_, _, _) => {
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

    fn bli(&mut self, a: u8, b: u8, mem: &mut Memory, io: &mut InputOutput) {
        match (a, b) {
            // LDI
            (4, 0) => {
                self.load_inc_dec(mem, true);       
            },
            // LDD
            (5, 0) => {
                self.load_inc_dec(mem, false);
            },
            // LDIR
            (6, 0) => {
                self.load_inc_dec(mem, true);
                let bc = self.reg.bc();
                // repeat if BC is not 0
                if bc != 0 {
                    self.load_inc_dec(mem, true);
                }
            },
            // LDDR
            (7, 0) => {
                self.load_inc_dec(mem, false);
                let bc = self.reg.bc();
                // repeat if BC is not 0
                if bc != 0 {
                    self.load_inc_dec(mem, false);
                }
            },
            // CPI
            (4, 1) => {
                self.comp_inc_dec(mem, true);
            },
            // CPD
            (5, 1) => {
                self.comp_inc_dec(mem, false);
            },
            // CPIR
            (6, 1) => {
                self.comp_inc_dec(mem, true);
                let bc = self.reg.bc();
                // repeat if BC is not 0
                if bc != 0 {
                    self.load_inc_dec(mem, false);
                }
            },
            // CPDIR
            (7, 1) => {
                self.comp_inc_dec(mem, false);
                let bc = self.reg.bc();
                // repeat if BC is not 0
                if bc != 0 {
                    self.load_inc_dec(mem, false);
                }
            },
            // INI
            (4, 2) => {
                mem.write_byte(io.input(self.reg.c), self.reg.hl());
                let tmp = self.reg.hl();
                self.reg.write_hl(tmp + 1);
                self.reg.b -= 1;
            },
            // IND
            (5, 2) => {
                mem.write_byte(io.input(self.reg.c), self.reg.hl());
                let tmp = self.reg.hl();
                self.reg.write_hl(tmp - 1);
                self.reg.b -= 1;
            },
            // INIR
            (6, 2) => {
                mem.write_byte(io.input(self.reg.c), self.reg.hl());
                let tmp = self.reg.hl();
                self.reg.write_hl(tmp + 1);
                self.reg.b -= 1;

                if self.reg.b != 0 {
                    mem.write_byte(io.input(self.reg.c), self.reg.hl());
                    let tmp = self.reg.hl();
                    self.reg.write_hl(tmp + 1);
                    self.reg.b -= 1;
                }
            },
            // INDR
            (7, 2) => {
                mem.write_byte(io.input(self.reg.c), self.reg.hl());
                let tmp = self.reg.hl();
                self.reg.write_hl(tmp - 1);
                self.reg.b -= 1;

                if self.reg.b != 0 {
                    mem.write_byte(io.input(self.reg.c), self.reg.hl());
                    let tmp = self.reg.hl();
                    self.reg.write_hl(tmp - 1);
                    self.reg.b -= 1;
                }
            },
            // OUTI
            (4, 3) => {
                io.output(self.reg.c, mem.read_byte(self.reg.hl()));
                let tmp = self.reg.hl();
                self.reg.write_hl(tmp + 1);
                self.reg.b -= 1;
            },
            // OUTD
            (5, 2) => {
                io.output(self.reg.c, mem.read_byte(self.reg.hl()));
                let tmp = self.reg.hl();
                self.reg.write_hl(tmp - 1);
                self.reg.b -= 1;
            },
            // OUTIR
            (6, 2) => {
                io.output(self.reg.c, mem.read_byte(self.reg.hl()));
                let tmp = self.reg.hl();
                self.reg.write_hl(tmp + 1);
                self.reg.b -= 1;

                if self.reg.b != 0 {
                    io.output(self.reg.c, mem.read_byte(self.reg.hl()));
                    let tmp = self.reg.hl();
                    self.reg.write_hl(tmp + 1);
                    self.reg.b -= 1;
                }
            },
            // OUTDR
            (7, 2) => {
                io.output(self.reg.c, mem.read_byte(self.reg.hl()));
                let tmp = self.reg.hl();
                self.reg.write_hl(tmp - 1);
                self.reg.b -= 1;

                if self.reg.b != 0 {
                    io.output(self.reg.c, mem.read_byte(self.reg.hl()));
                    let tmp = self.reg.hl();
                    self.reg.write_hl(tmp - 1);
                    self.reg.b -= 1;
                }
            },
            (_, _) => {}
        }
    }
    
    // Loads (HL) into (DE) and then increments or decrements HL and DE
    // based on the <inc> boolean. BC is always decremented
    fn load_inc_dec(&mut self, mem: &mut Memory, inc: bool) {
        let byte = mem.read_byte(self.reg.hl());
        mem.write_byte(byte, self.reg.de());
        
        let mut tmp = self.reg.de();
        if inc {
            // increment HL and DE
            self.reg.write_de(tmp + 1);
            tmp = self.reg.hl();
            self.reg.write_hl(tmp + 1);
        } else {
            // decrement HL and DE
            self.reg.write_de(tmp - 1);
            tmp = self.reg.hl();
            self.reg.write_hl(tmp - 1);
        }
        tmp = self.reg.bc();
        self.reg.write_bc(tmp - 1);
    }
    
    // Compares (HL) and A and increments or decrements HL based on <inc>
    // boolean. Again, BC is always decremented
    fn comp_inc_dec(&mut self, mem: &mut Memory, inc: bool) {
        let byte = mem.read_byte(self.reg.hl());
        let acc = self.reg.a;
        let result = byte - acc;
        self.reg.set_flag(1, true);
        self.detect_half_carry_sub(byte, acc, false);
        self.reg.set_flag(6, result == 0);
        self.reg.set_flag(7, result > 127);

        let mut tmp = self.reg.hl();
        if inc {
            // increment HL
            self.reg.write_hl(tmp + 1);
        } else {
            // decrement HL
            self.reg.write_hl(tmp - 1);
        }
        tmp = self.reg.bc();
        self.reg.write_bc(tmp - 1);
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

    fn add_16(&mut self, val: u16, sub_carry: bool) {
        let left = self.reg.hl();
        let right = val;
        let mut result = left.wrapping_add(right);

        if sub_carry {
            let carry = self.reg.read_flag(0);
            result = result.wrapping_add(carry as u16);
            self.detect_overflow_add(right as u8, left as u8, carry);
            self.detect_half_carry_add(right as u8, left as u8, carry);
        } else {
            self.detect_overflow_add(right as u8, left as u8, false);
            self.detect_half_carry_add(right as u8, left as u8, false);
        }
        self.reg.write_hl(result);

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

    fn sub_16(&mut self, val: u16, sub_carry: bool) {
        let left = self.reg.hl();
        let right = val;
        let mut result = left.wrapping_sub(right);

        if sub_carry {
            let carry = self.reg.read_flag(0);
            result = result.wrapping_sub(carry as u16);
            self.detect_overflow_sub(right as u8, left as u8, carry);
            self.detect_half_carry_sub(right as u8, left as u8, carry);
        } else {
            self.detect_overflow_sub(right as u8, left as u8, false);
            self.detect_half_carry_sub(right as u8, left as u8, false);
        }
        self.reg.write_hl(result);

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

    fn rot(&mut self, operator: u8, val: u8) -> u8{
        let mut val = val;
        match operator {
            // RLC
            0 => {self.shift(val, true, false);},
            // RRC
            1 => {self.shift(val, true, true);},
            // RL
            2 => {self.shift(val, false, false);},
            // RR
            3 => {self.shift(val, false, true);},
            // SLA
            4 => {
                self.reg.set_flag(0, (val & 1) == 0);
                val <<= 1;
            },
            // SRA
            5 => {
                self.reg.set_flag(0, (val & 1) == 0);
                // preserve 7th bit
                let bit_7 = val & (1 << 7);
                val = (val >> 1) | bit_7;
            },
            // SLL
            6 => {
                self.reg.set_flag(0, (val & 1) == 0);
                val <<= 1;
            },
            // SRL
            7 => {
                self.reg.set_flag(0, (val & 1) == 0);
                val >>= 1;
            }
            _ => {},
        }

        self.reg.set_flag(1, false);
        self.reg.set_flag(4, false);
        self.reg.set_flag(6, val == 0);
        self.reg.set_flag(7, val > 127);
        self.detect_parity(val);
        val
    }

    fn inc_pc(&mut self) {
        self.reg.pc += 1;
    }
    
    // adds d to pc
    fn jr(&mut self, d: i8) {
        let result = self.reg.pc as i16 + d as i16;
        if result < 0 {
            self.reg.pc = 0;
        } else {
            self.reg.pc = result as u16;
        }
        self.reg.pc += 2;
    }
    
    // used for simplifying RLCA, RLA, RRCA, RRA instructions
    //
    // if carry_bit is true, the 7th bit is carried over to the 0th,
    // otherwise the 0th is set to the carry flag before the instruction
    //
    // if right is true, a right shift is performed. left otherwise
    // TODO jesus christ what were you thinking, me?
    fn shift(&mut self, mut value: u8, carry_bit: bool, right: bool) -> u8 {
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
        
        self.reg.set_flag(0, carry);
        value
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

impl fmt::Debug for Z80 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.reg)
    }
}

#[cfg(test)]
mod tests;
