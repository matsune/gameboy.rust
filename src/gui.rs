use glium::texture::texture2d::Texture2d;
use glium::texture::{ClientFormat, MipmapsOption, RawImage2d, UncompressedFloatFormat};
use glium::{glutin, Surface};

use crate::gpu::{SCREEN_H, SCREEN_W};

const INIT_WINDOW_SCALE: usize = 2;

pub struct Window {
    events_loop: glutin::EventsLoop,
    display: glium::Display,
    texture: Texture2d,
}

impl Window {
    pub fn new(title: String) -> Self {
        let w = SCREEN_W as u32;
        let h = SCREEN_H as u32;
        let events_loop = glutin::EventsLoop::new();
        let window = glutin::WindowBuilder::new()
            .with_title(title)
            .with_dimensions((w * INIT_WINDOW_SCALE as u32, h * INIT_WINDOW_SCALE as u32).into());
        let context = glutin::ContextBuilder::new();
        let display = glium::Display::new(window, context, &events_loop).unwrap();
        let texture = Texture2d::empty_with_format(
            &display,
            UncompressedFloatFormat::U8U8U8,
            MipmapsOption::NoMipmap,
            w,
            h,
        )
        .unwrap();
        Self {
            events_loop,
            display,
            texture,
        }
    }

    pub fn draw(&self, data: Vec<u8>) {
        let w = SCREEN_W as u32;
        let h = SCREEN_H as u32;
        let rawimage2d = RawImage2d {
            data: std::borrow::Cow::Owned(data),
            width: w,
            height: h,
            format: ClientFormat::U8U8U8,
        };
        self.texture.write(
            glium::Rect {
                left: 0,
                bottom: 0,
                width: w,
                height: h,
            },
            rawimage2d,
        );

        let target = self.display.draw();
        let (target_w, target_h) = target.get_dimensions();
        self.texture.as_surface().blit_whole_color_to(
            &target,
            &glium::BlitTarget {
                left: 0,
                bottom: target_h,
                width: target_w as i32,
                height: -(target_h as i32),
            },
            glium::uniforms::MagnifySamplerFilter::Linear,
        );
        target.finish().unwrap();
    }

    pub fn poll_events<F>(&mut self, callback: F)
    where
        F: FnMut(glutin::Event),
    {
        self.events_loop.poll_events(callback);
    }
}
