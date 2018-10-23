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
                8
            }
            
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
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cpu::mem::TestMemory;

    #[test]
    fn test_run_opcodes() {
        let mut cpu = Z80::new();
        cpu.run_opcodes(5, &mut TestMemory::new());
        assert_eq!(cpu.reg.pc, 5);
    }

    #[test]
    fn test_run_opcode_16bit_immediate() {
        let mut cpu = Z80::new();
        let mut mem = TestMemory::new();

        mem.ram[0] = 0x01;
        mem.ram[1] = 0x30;

        cpu.run_opcode(0x01, &mut mem);
        assert_eq!(cpu.reg.bc(), 0x3000);
    }
}
