pub trait InputOutput {
    
    /// Read a byte from peripheral device at port <port>
    fn input(&self, port: u8) -> u8;

    /// Write byte <byte> to peripheral device at port <port>
    fn output(&mut self, port: u8, byte: u8);
}

/// An implementation of InputOutput trait for testing purposes
pub struct TestIO {
    pub data: u8
}

impl TestIO {
    pub fn new() -> Self {
        TestIO {
            data: 0
        }
    }
}

impl InputOutput for TestIO {
    fn input(&self, port: u8) -> u8 {
        self.data
    }

    fn output(&mut self, port: u8, byte: u8) {
        self.data = byte;
    }
}
