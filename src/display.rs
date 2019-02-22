use rom::Roms;

use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;

pub struct Display<'a> {
    roms: &'a Roms,
    frame_buffer: [[Color; 224]; 288],
}

impl<'a> Display<'a> {
    pub fn new(roms: &'a Roms) -> Self {
        Display {
            roms: roms,
            frame_buffer: [[Color::RGB(0, 0, 0); 224]; 288],
        }
    }

    pub fn draw_tile(&mut self, mut x: usize, mut y: usize, tile: usize, palette: usize) {
        x *= 8;
        y *= 8;
        let slice_index = tile * 64;

        self.draw_slice(x, y + 4, slice_index, palette, false, false, false);
        self.draw_slice(x, y, slice_index + 32, palette, false, false, false);
    }

    pub fn draw_sprite(&mut self, x: usize, y: usize, sprite: usize, palette: usize) {
        let slice_index = sprite * 256;

        self.draw_slice(x + 8, y + 12, slice_index, palette, true, false, false);
        self.draw_slice(x + 8, y, slice_index + 32, palette, true, false, false);
        self.draw_slice(x + 8, y + 4, slice_index + 64, palette, true, false, false);
        self.draw_slice(x + 8, y + 8, slice_index + 96, palette, true, false, false);
        self.draw_slice(x, y + 12, slice_index + 128, palette, true, false, false);
        self.draw_slice(x, y, slice_index + 160, palette, true, false, false);
        self.draw_slice(x, y + 4, slice_index + 192, palette, true, false, false);
        self.draw_slice(x, y + 8, slice_index + 224, palette, true, false, false);
    }

    pub fn show(&self, canvas: &mut Canvas<Window>) {
        for x in 0..224 {
            for y in 0..288 {
                let x_coord = (x * 2) as i32;
                let y_coord = (y * 2) as i32;
                canvas.set_draw_color(self.frame_buffer[y][x]);
                let result = canvas.fill_rect(Rect::new(x_coord, y_coord, 2, 2));
            }
        }
    }

    // draws an 8x4 slice of pixels. if sprite is true, it use sprite rom, tile rom otherwise
    fn draw_slice(
        &mut self,
        x: usize,
        y: usize,
        mut index: usize,
        palette: usize,
        sprite: bool,
        x_flip: bool,
        y_flip: bool,
    ) {
        let video_rom = &if sprite {
            self.roms.sprite_rom
        } else {
            self.roms.tile_rom
        };
        let palette = &self.roms.palette_rom[palette];

        for x_offset in (0..8).rev() {
            for y_offset in (0..4).rev() {
                let color_index = palette[video_rom[index] as usize];
                self.frame_buffer[y + y_offset][x + x_offset] = self.roms.color_rom[color_index];
                index += 1;
            }
        }
    }
}
