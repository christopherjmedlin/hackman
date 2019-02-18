mod rom;
mod memory_map;
mod memory_mapper;
mod cpu;
mod pacman;

#[macro_use(matches)]
extern crate matches;

use std::env;
use std::path::Path;
use rom::Roms;
use std::boxed::Box;
use pacman::PacmanSystem;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    let current_dir = env::current_dir().unwrap();
    let directory = match args.get(1) {
        Some(dir) => {Path::new(dir)},
        None => {current_dir.as_path()}
    };

    let rom = Box::new(Roms::load(&directory));
    let mut pacman = PacmanSystem::new(&rom);
    pacman.start();
}
