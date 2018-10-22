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

    fn run_opcode(&mut self, opcode: u8, memory: &mut Memory) {
        let immediate_8: u8 = memory.read_byte(self.reg.pc + 1);
        let immediate_16: u16 = (memory.read_byte(self.reg.pc + 1) as u16) << 8 | 
                                (memory.read_byte(self.reg.pc + 2) as u16);
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
