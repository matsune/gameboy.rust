use crate::cartridge::Cartridge;
use crate::cpu::CPU;
use crate::gui::Window;
use crate::joypad::JoypadKey;
use crate::memory::MMU;
use crate::sound::AudioPlayer;
use glium::glutin;
use std::path::Path;
use std::sync::mpsc::{channel, Receiver, Sender, TryRecvError};
use std::sync::{Arc, Mutex};
use std::thread;

pub struct Emulator<P: AsRef<Path>> {
    file_path: P,
    sav_path: Option<P>,
}

impl<P: AsRef<Path>> Emulator<P> {
    pub fn new(file_path: P) -> Self {
        Self {
            file_path,
            sav_path: None,
        }
    }

    pub fn sav_path(mut self, sav_path: Option<P>) -> Self {
        self.sav_path = sav_path;
        self
    }

    pub fn run(self, skip_boot: bool) {
        let (data_tx, data_rx) = channel();
        let (key_tx, key_rx) = channel();
        let mut gameboy = Gameboy::new(self.file_path, self.sav_path, skip_boot);
        let title = gameboy.mmu.title().to_owned();
        if let Some((player, event_loop, shared_buffer)) = CpalPlayer::new() {
            gameboy.mmu.enable_sound(Box::new(player));
            thread::spawn(move || Self::run_cpal_thread(event_loop, shared_buffer));
        }
        let cpu_thread = thread::Builder::new()
            .name("CPU thread".to_string())
            .spawn(move || Self::run_cpu_thread(gameboy, data_tx, key_rx))
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

    fn run_cpal_thread(
        event_loop: cpal::EventLoop,
        audio_buffer: Arc<Mutex<Vec<(f32, f32)>>>,
    ) -> ! {
        event_loop.run(move |_stream_id, stream_data| {
            let mut inbuffer = audio_buffer.lock().unwrap();
            if let cpal::StreamData::Output { buffer } = stream_data {
                let outlen = ::std::cmp::min(buffer.len() / 2, inbuffer.len());
                match buffer {
                    cpal::UnknownTypeOutputBuffer::F32(mut outbuffer) => {
                        for (i, (in_l, in_r)) in inbuffer.drain(..outlen).enumerate() {
                            outbuffer[i * 2] = in_l;
                            outbuffer[i * 2 + 1] = in_r;
                        }
                    }
                    cpal::UnknownTypeOutputBuffer::U16(mut outbuffer) => {
                        for (i, (in_l, in_r)) in inbuffer.drain(..outlen).enumerate() {
                            outbuffer[i * 2] = (in_l * f32::from(std::i16::MAX)
                                + f32::from(std::u16::MAX) / 2.0)
                                as u16;
                            outbuffer[i * 2 + 1] = (in_r * f32::from(std::i16::MAX)
                                + f32::from(std::u16::MAX) / 2.0)
                                as u16;
                        }
                    }
                    cpal::UnknownTypeOutputBuffer::I16(mut outbuffer) => {
                        for (i, (in_l, in_r)) in inbuffer.drain(..outlen).enumerate() {
                            outbuffer[i * 2] = (in_l * f32::from(std::i16::MAX)) as i16;
                            outbuffer[i * 2 + 1] = (in_r * f32::from(std::i16::MAX)) as i16;
                        }
                    }
                }
            }
        });
    }
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

struct Gameboy {
    pub cpu: CPU,
    pub mmu: MMU,
}

impl Gameboy {
    fn new<P: AsRef<Path>>(file_path: P, sav_path: Option<P>, skip_boot: bool) -> Self {
        let cartridge = Cartridge::new(file_path, sav_path, true).set_skip_boot(skip_boot);
        Self {
            cpu: CPU::new(skip_boot, cartridge.is_gbc),
            mmu: MMU::new(cartridge, skip_boot),
        }
    }

    fn tick(&mut self) {
        let cycles = self.cpu.tick(&mut self.mmu);
        self.mmu.tick(cycles * 4);
    }
}

struct CpalPlayer {
    buffer: Arc<Mutex<Vec<(f32, f32)>>>,
    sample_rate: u32,
}

impl CpalPlayer {
    fn new() -> Option<(CpalPlayer, cpal::EventLoop, Arc<Mutex<Vec<(f32, f32)>>>)> {
        let device = match cpal::default_output_device() {
            Some(e) => e,
            None => return None,
        };

        let mut wanted_samplerate = None;
        let mut wanted_sampleformat = None;
        let supported_formats = match device.supported_output_formats() {
            Ok(e) => e,
            Err(_) => return None,
        };
        for f in supported_formats {
            match wanted_samplerate {
                None => wanted_samplerate = Some(f.max_sample_rate),
                Some(cpal::SampleRate(r)) if r < f.max_sample_rate.0 && r < 192_000 => {
                    wanted_samplerate = Some(f.max_sample_rate)
                }
                _ => {}
            }
            match wanted_sampleformat {
                None => wanted_sampleformat = Some(f.data_type),
                Some(cpal::SampleFormat::F32) => {}
                Some(_) if f.data_type == cpal::SampleFormat::F32 => {
                    wanted_sampleformat = Some(f.data_type)
                }
                _ => {}
            }
        }

        if wanted_samplerate.is_none() || wanted_sampleformat.is_none() {
            return None;
        }

        let format = cpal::Format {
            channels: 2,
            sample_rate: wanted_samplerate.unwrap(),
            data_type: wanted_sampleformat.unwrap(),
        };

        let event_loop = cpal::EventLoop::new();
        let stream_id = event_loop.build_output_stream(&device, &format).unwrap();
        event_loop.play_stream(stream_id);

        let shared_buffer = Arc::new(Mutex::new(Vec::new()));
        let player = CpalPlayer {
            buffer: shared_buffer.clone(),
            sample_rate: wanted_samplerate.unwrap().0,
        };

        Some((player, event_loop, shared_buffer))
    }
}

impl AudioPlayer for CpalPlayer {
    fn play(&mut self, buf_left: &[f32], buf_right: &[f32]) {
        debug_assert!(buf_left.len() == buf_right.len());

        let mut buffer = self.buffer.lock().unwrap();

        for (l, r) in buf_left.iter().zip(buf_right) {
            if buffer.len() > self.sample_rate as usize {
                return;
            }
            buffer.push((*l, *r));
        }
    }

    fn samples_rate(&self) -> u32 {
        self.sample_rate
    }

    fn underflowed(&self) -> bool {
        (*self.buffer.lock().unwrap()).is_empty()
    }
}
