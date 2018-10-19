use std::path::Path;
use std::fs::File;
use std::io::Read;

const ROM_FILE_NAMES: [&str; 10] = [
    "pacman.6e",
    "pacman.6f",
    "pacman.6h",
    "pacman.6j",
    "82s123.7f",
    "82s123.4a",
    "pacman.5e",
    "pacman.5f",
    "82s126.1m",
    "82s126.3m"
];

pub struct Roms {
    pub game_roms: [[u8; 4096]; 4],
    pub color_rom: [u8; 32],
    pub palette_rom: [u8; 256],
    pub tile_rom: [u8; 4096],
    pub sprite_rom: [u8; 4096],
    pub sound_roms: [[u8; 256]; 2],
}

impl Roms {
    pub fn new() -> Self {
        Roms {
            game_roms: [[0; 4096]; 4],
            color_rom: [0; 32],
            palette_rom: [0; 256],
            tile_rom: [0; 4096],
            sprite_rom: [0; 4096],
            sound_roms: [[0; 256]; 2]
        }
    }

    pub fn load(directory: &str) -> Self {
        let mut roms = Roms::new();

        for file_name in ROM_FILE_NAMES.iter() {
            let path = Path::new(directory).join(file_name);

            let mut file = match File::open(&path) {
                Err(why) => panic!("Missing ROMs"),
                Ok(file) => file,
            };
            
            let buf: &mut [u8];
            buf = match *file_name {
                "pacman.6e" => &mut roms.game_roms[0],
                "pacman.6f" => &mut roms.game_roms[1],
                "pacman.6h" => &mut roms.game_roms[2],
                "pacman.6j" => &mut roms.game_roms[3],
                "82s123.7f" => &mut roms.color_rom,
                "82s126.4a" => &mut roms.palette_rom,
                "pacman.5e" => &mut roms.tile_rom,
                "pacman.5f" => &mut roms.sprite_rom,
                "82s126.1m" => &mut roms.sound_roms[0],
                "82s126.3m" => &mut roms.sound_roms[1],
                _ => panic!("Internal error")
            };

            match file.read(buf) {
                Ok(_) => {},
                Err(why) => panic!("Error reading ROM")
            }
        }

        roms
    }
}
