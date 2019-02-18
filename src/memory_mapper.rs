use memory_map::{Address, map_address};
use cpu::mem::Memory;
use rom::Roms;

pub struct MemoryMapper<'a> {
    roms: &'a Box<Roms>,
    ram: [u8; 2032],
}

impl<'a> MemoryMapper<'a> {
    pub fn new(roms: &'a Box<Roms>) -> Self {
        MemoryMapper {
            roms: roms,
            ram: [0; 2032]
        }
    }

    fn map(addr: u16, writing: bool) -> Address {
        match map_address(addr, writing) {
            Ok(addr) => addr,
            Err(why) => panic!("Failed to map memory address: {}", why),
        }
    }
}

impl<'a> Memory for MemoryMapper<'a> {

    fn write_byte(&mut self, byte: u8, addr: u16) {
        match MemoryMapper::map(addr, true) {
            Address::Ram(offset) => {self.ram[offset] = byte;},
            _ => {}
        }
    }

    fn read_byte(&self, addr: u16) -> u8 {
        match MemoryMapper::map(addr, false) {
            Address::GameRom(offset) =>
                self.roms.game_roms[offset/1000][offset%1000],

            Address::Ram(offset) => self.ram[offset],
            _ => 0,
        }
    }
}

mod tests {
    use super::*;
    use std::boxed::Box;

    #[test]
    fn test_inital_state() {
        let roms = Box::new(Roms::new());
        let mapper = MemoryMapper::new(&roms);

        assert_eq!(mapper.ram[2], 0);
        assert_eq!(mapper.roms.game_roms[0][0], 0);
    }

    #[test]
    fn test_valid_write() {
        let roms = Box::new(Roms::new());
        let mut mapper = MemoryMapper::new(&roms);

        mapper.write_byte(0x1, 0x4803);
        assert_eq!(mapper.ram[0x3], 0x1);
    }

    #[test]
    fn test_read() {
        let roms = Box::new(Roms::new());
        let mut mapper = MemoryMapper::new(&roms);

        mapper.ram[0x3] = 0x1;
        assert_eq!(mapper.read_byte(0x4803), 0x1);
    }


    #[test]
    #[should_panic]
    fn test_invalid_write() {
        let roms = Box::new(Roms::new());
        let mut mapper = MemoryMapper::new(&roms);
        mapper.write_byte(0x11, 0x01); 
    }
}
