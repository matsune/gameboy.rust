use crate::cartridge::Cartridge;
use crate::gb::Gameboy;
use crate::window::Window;

pub struct Emulator {
    gameboy: Gameboy,
    window: Window,
}

impl Emulator {
    pub fn new(memory: Vec<u8>) -> Self {
        Self {
            gameboy: Gameboy::new(Cartridge::new(memory)),
            window: Window::new(),
        }
    }

    pub fn run(mut self) {
        use glium::glutin;

        let mut closed = false;
        while !closed {
            self.gameboy.tick();

            if self.gameboy.mmu.gpu.redraw {
                self.window.draw(&self.gameboy.mmu.gpu.data);
            }

            self.window.poll_events(|event| match event {
                glutin::Event::WindowEvent { event, .. } => match event {
                    glutin::WindowEvent::CloseRequested => closed = true,
                    _ => (),
                },
                _ => (),
            });
        }
    }
}
