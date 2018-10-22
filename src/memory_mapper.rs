use memory_map::{Address, map_address};
use cpu::mem::Memory;
use rom::Roms;

pub struct MemoryMapper {
    roms: Roms,
    ram: [u8; 2032],
}

impl MemoryMapper {
    pub fn new(roms: Roms) -> Self {
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

impl Memory for MemoryMapper {

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

    #[test]
    fn test_inital_state() {
        let mapper = MemoryMapper::new(Roms::new());

        assert_eq!(mapper.ram[2], 0);
        assert_eq!(mapper.roms.game_roms[0][0], 0);
    }

    #[test]
    fn test_valid_write() {
        let mut mapper = MemoryMapper::new(Roms::new());
        
        mapper.write_byte(0x1, 0x4803);
        assert_eq!(mapper.ram[0x3], 0x1);
    }

    #[test]
    fn test_read() {
        let mut mapper = MemoryMapper::new(Roms::new());

        mapper.ram[0x3] = 0x1;
        assert_eq!(mapper.read_byte(0x4803), 0x1);
    }


    #[test]
    #[should_panic]
    fn test_invalid_write() {
        let mut mapper = MemoryMapper::new(Roms::new());

        mapper.write_byte(0x11, 0x01); 
    }
}
