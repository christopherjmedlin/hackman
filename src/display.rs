use rom::Roms;

use sdl2::render::Canvas;
use sdl2::video::Window;
use sdl2::rect::Rect;

pub struct Display<'a> {
    roms: &'a Roms,
    frame_buffer: [[u8; 224]; 288],
}

impl<'a> Display<'a> {
    pub fn new(roms: &'a Roms) -> Self {
        Display {
            roms: roms,
            frame_buffer: [[0; 224]; 288],
        }
    }

    pub fn draw_tile(&mut self, mut x: usize, mut y: usize, tile: usize) {
        x *= 8;
        y *= 8;
        let slice_index = tile * 64;

        self.draw_slice(x, y + 4, slice_index, false);
        self.draw_slice(x, y, slice_index + 32, false);
    }

    pub fn show(&self, canvas: &mut Canvas<Window>) {
        for x in 0..224  {
            for y in 0..288 {
                if self.frame_buffer[y][x] != 0 {
                    let x_coord = (x * 4) as i32;
                    let y_coord = (y * 4) as i32;
                    let result = canvas.fill_rect(Rect::new(x_coord, y_coord, 4, 4));
                }
            }
        }
    }
    
    // draws an 8x4 slice of pixels. if sprite is true, it use sprite rom, tile rom otherwise
    fn draw_slice(&mut self, x: usize, y: usize, mut index: usize, sprite: bool) {
        let rom = & if sprite {self.roms.tile_rom} else {self.roms.tile_rom};

        for x_offset in (0..8).rev() {
            for y_offset in (0..4).rev() {
                self.frame_buffer[y + y_offset][x + x_offset] = rom[index];
                index += 1;
            }
        }
    }
}
