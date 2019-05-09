use glium::glutin;
use std::sync::mpsc::{channel, Sender};
use std::thread;

use crate::cartridge::Cartridge;
use crate::gb::Gameboy;
use crate::window::Window;

fn run_cpu_thread(mut gameboy: Gameboy, data_tx: Sender<Vec<u8>>) {
    loop {
        if gameboy.tick() {
            let data = gameboy.mmu.gpu.data.clone();
            if data_tx.send(data).is_err() {
                break;
            }
        }
    }
}

pub fn run(cartridge: Cartridge) {
    let (data_tx, data_rx) = channel();
    let gameboy = Gameboy::new(cartridge);
    let cpu_thread = thread::spawn(move || run_cpu_thread(gameboy, data_tx));

    let mut window = Window::new();
    let mut closed = false;
    while !closed {
        if let Ok(data) = data_rx.try_recv() {
            window.draw(data);
        }

        window.poll_events(|event| match event {
            glutin::Event::WindowEvent { event, .. } => match event {
                glutin::WindowEvent::CloseRequested => closed = true,
                _ => (),
            },
            _ => (),
        });
    }
    drop(data_rx);
    cpu_thread.join().unwrap();
}
