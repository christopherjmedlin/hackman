mod cpu;
mod display;
mod interrupt_vector;
mod memory_map;
mod memory_mapper;
mod pacman;
mod rom;

#[macro_use(matches)]
extern crate matches;
extern crate sdl2;

use pacman::PacmanSystem;
use rom::Roms;
use std::boxed::Box;
use std::env;
use std::path::Path;

fn main() {
    let args: Vec<String> = env::args().collect();

    let current_dir = env::current_dir().unwrap();
    let directory = match args.get(1) {
        Some(dir) => Path::new(dir),
        None => current_dir.as_path(),
    };

    let rom = Box::new(Roms::load(&directory));
    let mut pacman = PacmanSystem::new(&rom);

    if args.contains(&String::from("--debug")) {
        pacman.debug();
    } else {
        pacman.start();
    }
}
