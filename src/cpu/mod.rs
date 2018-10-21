mod reg;

use memory_mapper::MemoryMapper;
use cpu::reg::Registers;

pub struct Z80 {
    reg: Registers,
    altreg: Registers,

    ix: u16,
    iy: u16,
    sp: u16,

    i: u8,
    r: u8,

    pc: u16,

    s: bool,
    z: bool,
    h: bool,
    p: bool,
    n: bool,
    c: bool,
}

impl Z80 {
    pub fn new() -> Self {
        Z80 {
            reg: Registers::new(),
            altreg: Registers::new(),

            ix: 0,
            iy: 0,
            sp: 0,

            i: 0,
            r: 0,

            pc: 0,

            s: false,
            z: false,
            h: false,
            p: false,
            n: false,
            c: false
        }
    }
    
    /// Runs a specified number of opcodes
    pub fn run_opcodes(&mut self, iters: usize, memory: &mut MemoryMapper) {
        for i in 0..iters {
            let opcode = memory.read_byte(self.pc);
            self.run_opcode(opcode, memory);
            self.pc += 1;
        }
    }

    fn run_opcode(&mut self, opcode: u8, memory: &mut MemoryMapper) {
        let immediate_8: u8 = memory.read_byte(self.pc + 1);
        let immediate_16: u16 = (memory.read_byte(self.pc) as u16) << 8 | 
                                (memory.read_byte(self.pc + 1) as u16);
        match opcode {
            0x00 => {},
            0x01 => {self.reg.bc = immediate_16},
            0x02 => {memory.write_byte(self.reg.a(), self.reg.bc)},
            _ => {println!("WARNING: unimplemented opcode 0x{:x}", opcode)}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rom::Roms;

    #[test]
    fn test_run_opcode_16bit_immediate() {
        let mut cpu = Z80::new();
        let mut mem = MemoryMapper::new(Roms::new());

        cpu.run_opcode(0x012000, &mut mem);
        assert_eq!(cpu.reg.bc, 0x2000);
    }
}
