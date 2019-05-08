use glium::texture::texture2d::Texture2d;
use glium::texture::{ClientFormat, MipmapsOption, RawImage2d, UncompressedFloatFormat};
use glium::{glutin, Surface};

use crate::gpu::{PIXELS_H, PIXELS_W};

const INIT_WINDOW_SCALE: u32 = 3;

pub struct Window {
    events_loop: glutin::EventsLoop,
    display: glium::Display,
    texture: Texture2d,
}

impl Window {
    pub fn new() -> Self {
        let w = u32::from(PIXELS_W);
        let h = u32::from(PIXELS_H);
        let events_loop = glutin::EventsLoop::new();
        let window = glutin::WindowBuilder::new()
            .with_dimensions((w * INIT_WINDOW_SCALE, h * INIT_WINDOW_SCALE).into());
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

    pub fn draw(&self, data: &[u8]) {
        let w = u32::from(PIXELS_W);
        let h = u32::from(PIXELS_H);
        let rawimage2d = RawImage2d {
            data: std::borrow::Cow::Borrowed(data),
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
