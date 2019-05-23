use crate::cartridge::Cartridge;
use crate::gb::Gameboy;
use crate::joypad::JoypadKey;
use crate::window::Window;
use glium::glutin;
use std::sync::mpsc::{channel, Receiver, Sender, TryRecvError};
use std::thread;

fn run_cpu_thread(
    mut gameboy: Gameboy,
    data_tx: Sender<Vec<u8>>,
    key_rx: Receiver<(glutin::ElementState, JoypadKey)>,
) {
    'main: loop {
        gameboy.tick();
        if gameboy.mmu.gpu.redraw {
            gameboy.mmu.gpu.redraw = false;
            let data = gameboy.mmu.gpu.get_rgb_data();
            if data_tx.send(data).is_err() {
                break 'main;
            }
        }

        'try_key: loop {
            match key_rx.try_recv() {
                Ok((state, key)) => match state {
                    glutin::ElementState::Pressed => gameboy.mmu.keydown(key),
                    glutin::ElementState::Released => gameboy.mmu.keyup(key),
                },
                Err(err) => match err {
                    TryRecvError::Disconnected => break 'main,
                    TryRecvError::Empty => break 'try_key,
                },
            }
        }
    }
}

pub fn run(data: Vec<u8>, skip_boot: bool) {
    let (data_tx, data_rx) = channel();
    let (key_tx, key_rx) = channel();
    let gameboy = Gameboy::new(Cartridge::new(data, skip_boot, Option::None));
    let title = gameboy.mmu.title().to_owned();
    let cpu_thread = thread::Builder::new()
        .name("CPU thread".to_string())
        .spawn(move || run_cpu_thread(gameboy, data_tx, key_rx))
        .unwrap();

    let mut window = Window::new(title);
    let mut closed = false;
    while !closed {
        if let Ok(data) = data_rx.try_recv() {
            window.draw(data);
        }
        match data_rx.try_recv() {
            Ok(data) => window.draw(data),
            Err(err) => match err {
                TryRecvError::Disconnected => break,
                TryRecvError::Empty => {}
            },
        }

        window.poll_events(|event| {
            closed = match event {
                glutin::Event::WindowEvent { event, .. } => match event {
                    glutin::WindowEvent::CloseRequested => true,
                    glutin::WindowEvent::KeyboardInput { input, .. } => {
                        match input.virtual_keycode {
                            Some(key) => match get_joypad_key(key) {
                                Some(key) => key_tx.send((input.state, key)).is_err(),
                                None => false,
                            },
                            None => false,
                        }
                    }
                    _ => false,
                },
                _ => false,
            }
        });
    }
    drop(data_rx);
    cpu_thread.join().unwrap();
}

fn get_joypad_key(key: glutin::VirtualKeyCode) -> Option<JoypadKey> {
    match key {
        glutin::VirtualKeyCode::Up => Some(JoypadKey::Up),
        glutin::VirtualKeyCode::Down => Some(JoypadKey::Down),
        glutin::VirtualKeyCode::Left => Some(JoypadKey::Left),
        glutin::VirtualKeyCode::Right => Some(JoypadKey::Right),
        glutin::VirtualKeyCode::Return => Some(JoypadKey::Start),
        glutin::VirtualKeyCode::Space => Some(JoypadKey::Select),
        glutin::VirtualKeyCode::Z => Some(JoypadKey::A),
        glutin::VirtualKeyCode::X => Some(JoypadKey::B),
        _ => None,
    }
}
