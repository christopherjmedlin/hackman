use cpu::mem::Memory;
use display::Display;
use memory_map::{map_address, Address};
use rom::Roms;

pub struct MemoryMapper<'a> {
    roms: &'a Box<Roms>,
    ram: [u8; 2032],
    tile_ram: [usize; 0x400],
    palette_ram: [usize; 0x400],
}

impl<'a> MemoryMapper<'a> {
    pub fn new(roms: &'a Box<Roms>) -> Self {
        MemoryMapper {
            roms: roms,
            ram: [0; 2032],
            tile_ram: [0; 0x400],
            palette_ram: [0; 0x400],
        }
    }

    fn map(addr: u16, writing: bool) -> Address {
        match map_address(addr, writing) {
            Ok(addr) => addr,
            Err(why) => panic!("Failed to map memory address: 0x{:x} ({})", addr, why),
        }
    }

    pub fn render(&self, display: &mut Display) {
        let mut addr = 0x3A0;
        for x in 2..30 {
            for y in 2..34 {
                display.draw_tile(x - 2, y - 2, self.tile_ram[addr], self.palette_ram[addr]);
                addr -= 1;
            }
        }
    }
}

impl<'a> Memory for MemoryMapper<'a> {
    fn write_byte(&mut self, byte: u8, addr: u16) {
        match MemoryMapper::map(addr, true) {
            Address::Ram(offset) => {
                self.ram[offset] = byte;
            }
            Address::VramTiles(offset) => {
                self.tile_ram[offset] = byte as usize;
            }
            Address::VramPalettes(offset) => {
                self.palette_ram[offset] = byte as usize;
            }
            _ => {}
        }
    }

    fn read_byte(&self, addr: u16) -> u8 {
        match MemoryMapper::map(addr, false) {
            Address::GameRom(offset) => self.roms.game_roms[offset / 0x1000][offset % 0x1000],

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
