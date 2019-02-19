use rom::Roms;
use cpu::Z80;
use cpu::io::TestIO;
use cpu::mem::Memory;
use memory_mapper::MemoryMapper;
use std::io;

pub struct PacmanSystem<'a> {
    roms: &'a Box<Roms>,
    cpu: Z80,
    memory: MemoryMapper<'a>,
    // just for now
    io: TestIO
}

impl<'a> PacmanSystem<'a> {
    pub fn new(roms: &'a Box<Roms>) -> Self {
        PacmanSystem {
            roms: roms,
            cpu: Z80::new(),
            memory: MemoryMapper::new(roms),
            io: TestIO::new()
        }
    }

    pub fn start(&mut self) {
        while true {
            self.cpu.run_opcodes(1, &mut self.memory, &mut self.io);
        }
    }

    pub fn debug(&mut self) {
        let mut input = String::new();
        while true {
            let pc = self.cpu.get_pc();
            println!("opcode: {:x}", self.memory.read_byte(pc));
            println!("{:?}", self.cpu);
            self.cpu.run_opcodes(1, &mut self.memory, &mut self.io);
            io::stdin().read_line(&mut input);
        }
    }
}
