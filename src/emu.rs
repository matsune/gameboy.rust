use glium::glutin;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

use crate::cartridge::Cartridge;
use crate::gb::Gameboy;
use crate::joypad::JoypadKey;
use crate::window::Window;

fn run_cpu_thread(
    mut gameboy: Gameboy,
    data_tx: Sender<Vec<u8>>,
    key_rx: Receiver<(glutin::ElementState, JoypadKey)>,
) {
    loop {
        if let Ok((state, key)) = key_rx.try_recv() {
            match state {
                glutin::ElementState::Pressed => gameboy.mmu.keydown(key),
                glutin::ElementState::Released => gameboy.mmu.keyup(key),
            }
        }
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
    let (key_tx, key_rx) = channel();
    let gameboy = Gameboy::new(cartridge);
    let cpu_thread = thread::spawn(move || run_cpu_thread(gameboy, data_tx, key_rx));

    let mut window = Window::new();
    let mut closed = false;
    while !closed {
        if let Ok(data) = data_rx.try_recv() {
            window.draw(data);
        }

        window.poll_events(|event| match event {
            glutin::Event::WindowEvent { event, .. } => match event {
                glutin::WindowEvent::CloseRequested => closed = true,
                glutin::WindowEvent::KeyboardInput { input, .. } => match input.state {
                    glutin::ElementState::Pressed => match input.virtual_keycode {
                        Some(glutin::VirtualKeyCode::Up) => {
                            key_tx.send((input.state, JoypadKey::Up));
                        }
                        Some(glutin::VirtualKeyCode::Down) => {
                            key_tx.send((input.state, JoypadKey::Down));
                        }
                        Some(glutin::VirtualKeyCode::Left) => {
                            key_tx.send((input.state, JoypadKey::Left));
                        }
                        Some(glutin::VirtualKeyCode::Right) => {
                            key_tx.send((input.state, JoypadKey::Right));
                        }
                        Some(glutin::VirtualKeyCode::Return) => {
                            key_tx.send((input.state, JoypadKey::Start));
                        }
                        Some(glutin::VirtualKeyCode::Space) => {
                            key_tx.send((input.state, JoypadKey::Select));
                        }
                        Some(glutin::VirtualKeyCode::Z) => {
                            key_tx.send((input.state, JoypadKey::A));
                        }
                        Some(glutin::VirtualKeyCode::X) => {
                            key_tx.send((input.state, JoypadKey::B));
                        }
                        _ => (),
                    },
                    glutin::ElementState::Released => match input.virtual_keycode {
                        Some(glutin::VirtualKeyCode::Up) => {
                            key_tx.send((input.state, JoypadKey::Up));
                        }
                        Some(glutin::VirtualKeyCode::Down) => {
                            key_tx.send((input.state, JoypadKey::Down));
                        }
                        Some(glutin::VirtualKeyCode::Left) => {
                            key_tx.send((input.state, JoypadKey::Left));
                        }
                        Some(glutin::VirtualKeyCode::Right) => {
                            key_tx.send((input.state, JoypadKey::Right));
                        }
                        Some(glutin::VirtualKeyCode::Return) => {
                            key_tx.send((input.state, JoypadKey::Start));
                        }
                        Some(glutin::VirtualKeyCode::Space) => {
                            key_tx.send((input.state, JoypadKey::Select));
                        }
                        Some(glutin::VirtualKeyCode::Z) => {
                            key_tx.send((input.state, JoypadKey::A));
                        }
                        Some(glutin::VirtualKeyCode::X) => {
                            key_tx.send((input.state, JoypadKey::B));
                        }
                        _ => (),
                    },
                },
                _ => (),
            },
            _ => (),
        });
    }
    drop(data_rx);
    cpu_thread.join().unwrap();
}
