use rom::Roms;

pub struct Display<'a> {
    roms: &'a Roms
}

impl<'a> Display<'a> {
    pub fn new(roms: &'a Roms) -> Self {
        Display {
            roms: roms
        }
    }
    
    // draws an 8x4 slice of pixels. if sprite is true, it use sprite rom, tile rom otherwise
    pub fn draw_slice(x: usize, y: usize, mut index: usize, sprite: bool) {
        for x_offset in 0..8.rev() {
            for y_offset in 0..4.rev() {
                // draw
            }
        }
    }
}
