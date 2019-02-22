use cpu::io::InputOutput;

pub struct InterruptVector {
    pub data: u8,
}

impl InterruptVector {
    pub fn new() -> Self {
        InterruptVector { data: 0 }
    }
}

impl InputOutput for InterruptVector {
    fn input(&self, port: u8) -> u8 {
        self.data
    }

    fn output(&mut self, port: u8, byte: u8) {
        self.data = byte;
    }
}
