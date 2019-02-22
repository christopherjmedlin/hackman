use cpu::mem::Memory;
use cpu::Z80;
use display::Display;
use interrupt_vector::InterruptVector;
use memory_mapper::MemoryMapper;
use rom::Roms;

use sdl2;
use sdl2::event::Event;
use std::io;

pub struct PacmanSystem<'a> {
    roms: &'a Box<Roms>,
    cpu: Z80,
    memory: MemoryMapper<'a>,
    // just for now
    io: InterruptVector,
    display: Display<'a>,
}

impl<'a> PacmanSystem<'a> {
    pub fn new(roms: &'a Box<Roms>) -> Self {
        PacmanSystem {
            roms: roms,
            cpu: Z80::new(),
            memory: MemoryMapper::new(roms),
            io: InterruptVector::new(),
            display: Display::new(roms),
        }
    }

    pub fn start(&mut self) {
        /*
        while true {
        }
        */

        let sdl_context = sdl2::init().unwrap();
        let video = sdl_context.video().unwrap();
        let window = video
            .window("Pacman", 488, 576)
            .position_centered()
            .build()
            .unwrap();

        let mut event_pump = sdl_context.event_pump().unwrap();
        let mut canvas = window.into_canvas().build().unwrap();

        let mut cycles = 0;
        'main: loop {
            cycles += self.cpu.run_opcodes(5, &mut self.memory, &mut self.io);
            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit { .. } => break 'main,
                    _ => {}
                }
            }

            self.memory.render(&mut self.display);
            canvas.present();

            //println!("{}", self.io.data);
            if cycles > 51200 {
                cycles = 0;
                self.memory.render(&mut self.display);
                self.display.show(&mut canvas);
                canvas.present();
                self.cpu.interrupt(self.io.data);
            }
        }
    }

    pub fn debug(&mut self) {
        let mut input = String::new();
        // break at the beginning
        let mut break_point: u16 = 0;
        let mut step = false;

        while true {
            input.clear();
            let pc = self.cpu.get_pc();

            if step || pc == break_point {
                let result = io::stdin().read_line(&mut input);
                match result {
                    Ok(n) => {
                        println!("{}", n);
                    }
                    Err(error) => {
                        println!("{}", error);
                    }
                }
                let split: Vec<&str> = input.split(" ").collect();
                match split.get(0) {
                    Some(result) => {
                        println!("{}", *result);
                        match *result {
                            "b" | "break" => {
                                break_point = match split.get(1) {
                                    Some(addr) => addr.parse().unwrap(),
                                    None => 0,
                                }
                            }
                            "s" | "step" => {
                                println!("asdf");
                                step = true;
                            }
                            &_ => {
                                println!("Invalid command");
                            }
                        }
                    }
                    None => {}
                }
                println!("opcode: {:x}", self.memory.read_byte(pc));
                println!("{:?}", self.cpu);
            }

            self.cpu.run_opcodes(1, &mut self.memory, &mut self.io);
        }
    }
}
