use rom::Roms;
use cpu::Z80;
use cpu::io::TestIO;
use memory_mapper::MemoryMapper;

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
}
