use std::path::Path;
use std::fs::File;
use std::io::Read;
use sdl2::pixels::Color;

const GAME_ROM_FILE_NAMES: [&str; 4] = [
    "pacman.6e",
    "pacman.6f",
    "pacman.6h",
    "pacman.6j",
];
const COLOR_ROM_FILE_NAME: &str = "82s123.7f";
const PALETTE_ROM_FILE_NAME: &str = "82s126.4a";
const TILE_ROM_FILE_NAME: &str = "pacman.5e";

pub struct Roms {
    pub game_roms: [[u8; 4096]; 4],
    pub color_rom: [Color; 32],
    // usize because palettes just contain indices into the color_rom
    pub palette_rom: [[usize; 4]; 64],
    pub tile_rom: [u8; 16384],
    pub sprite_rom: [u8; 4096],
    pub sound_roms: [[u8; 256]; 2],
}

impl Roms {
    pub fn new() -> Self {
        Roms {
            game_roms: [[0; 4096]; 4],
            color_rom: [Color {r: 0, g: 0, b: 0, a: 0}; 32],
            palette_rom: [[0; 4]; 64],
            tile_rom: [0; 16384],
            sprite_rom: [0; 4096],
            sound_roms: [[0; 256]; 2]
        }
    }

    pub fn load(directory: &Path) -> Self {
        let mut roms = Roms::new();

        roms.load_game_roms(directory);
        roms.load_color_rom(directory);
        roms.load_palette_rom(directory);
        roms.load_tile_rom(directory);
        roms
    }

    fn load_game_roms(&mut self, directory: &Path) {
        for file_name in GAME_ROM_FILE_NAMES.iter() {
            let path = directory.join(file_name);
            let mut buf: &mut [u8];
            buf = match *file_name {
                "pacman.6e" => &mut self.game_roms[0],
                "pacman.6f" => &mut self.game_roms[1],
                "pacman.6h" => &mut self.game_roms[2],
                "pacman.6j" => &mut self.game_roms[3],
                _ => panic!("Internal error")
            };
            Roms::load_file(&path, &mut buf);
        };
    }

    fn load_color_rom(&mut self, directory: &Path) {
        let mut bytes: [u8; 32] = [0; 32];
        Roms::load_file(&directory.join(COLOR_ROM_FILE_NAME), &mut bytes);

        for (i, byte) in bytes.iter().enumerate() {
            let mut color = &mut self.color_rom[i];
            if (byte & 1) != 0 {color.r += 0x21}
            if (byte & 1 << 1) != 0 {color.r += 0x47}
            if (byte & 1 << 2) != 0 {color.r += 0x97}
            if (byte & 1 << 3) != 0 {color.g += 0x21}
            if (byte & 1 << 4) != 0 {color.g += 0x47}
            if (byte & 1 << 5) != 0 {color.g += 0x97}
            if (byte & 1 << 6) != 0 {color.b += 0x51}
            if (byte & 1 << 7) != 0 {color.b += 0xAE}
        }
    }

    fn load_palette_rom(&mut self, directory: &Path) {
        let mut bytes: [u8; 256] = [0; 256];
        Roms::load_file(&directory.join(PALETTE_ROM_FILE_NAME), &mut bytes);

        for (i, byte) in bytes.iter().enumerate() {
            self.palette_rom[i/4][i%4] = *byte as usize;
            let color = self.color_rom[*byte as usize];
        }
    }
    
    // decodes the bit planes
    fn load_tile_rom(&mut self, directory: &Path) {
        let mut bytes: [u8; 4096] = [0; 4096];
        Roms::load_file(&directory.join(TILE_ROM_FILE_NAME), &mut bytes);
        
        for (i, byte) in bytes.iter().enumerate() {
            for bit in 0..4 {
                let lsb = (byte & 1 << bit) >> bit;
                let msb = (byte & 1 << (bit + 4)) >> (bit + 3);
                self.tile_rom[i * 4 + bit] = lsb | msb;
            }
        }
    }

    fn load_file(path: &Path, buffer: &mut [u8]) {
        let mut file = match File::open(&path) {
            Err(why) => panic!("Missing ROMs"),
            Ok(file) => file,
        };
        match file.read(buffer) {
                Ok(_) => {},
                Err(why) => panic!("Error reading ROM")
        };
    }
}
