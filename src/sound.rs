use crate::memory::Memory;
use crate::util::is_bit_on;
use blip_buf::BlipBuf;

const WAVE_PATTERN: [[i32; 8]; 4] = [
    [-1, -1, -1, -1, 1, -1, -1, -1],
    [-1, -1, -1, -1, 1, 1, -1, -1],
    [-1, -1, 1, 1, 1, 1, -1, -1],
    [1, 1, 1, 1, -1, -1, 1, 1],
];
const CLOCKS_PER_SECOND: u32 = 1 << 22;
const OUTPUT_SAMPLE_COUNT: usize = 2000;

pub trait AudioPlayer: Send {
    fn play(&mut self, left_channel: &[f32], right_channel: &[f32]);
    fn samples_rate(&self) -> u32;
    fn underflowed(&self) -> bool;
}

pub struct Sound {
    channel1: SquareSound,
    channel2: SquareSound,
    channel3: WaveSound,
    channel4: NoiseSound,
    volume_left: u8,
    volume_right: u8,
    on: bool,
    time: u32,
    output_period: u32,
    prev_time: u32,
    next_time: u32,
    need_sync: bool,
    time_divider: u8,
    player: Box<dyn AudioPlayer>,
    nr51: u8,
}

fn create_blipbuf(samples_rate: u32) -> BlipBuf {
    let mut blipbuf = BlipBuf::new(samples_rate);
    blipbuf.set_rates(f64::from(CLOCKS_PER_SECOND), f64::from(samples_rate));
    blipbuf
}

impl Sound {
    pub fn new(player: Box<dyn AudioPlayer>) -> Self {
        let blipbuf1 = create_blipbuf(player.samples_rate());
        let blipbuf2 = create_blipbuf(player.samples_rate());
        let blipbuf3 = create_blipbuf(player.samples_rate());
        let blipbuf4 = create_blipbuf(player.samples_rate());

        let output_period = (OUTPUT_SAMPLE_COUNT as u64 * u64::from(CLOCKS_PER_SECOND))
            / u64::from(player.samples_rate());
        Sound {
            channel1: SquareSound::new(blipbuf1, true),
            channel2: SquareSound::new(blipbuf2, false),
            channel3: WaveSound::new(blipbuf3),
            channel4: NoiseSound::new(blipbuf4),
            volume_left: 7,
            volume_right: 7,
            on: false,
            time: 0,
            output_period: output_period as u32,
            prev_time: 0,
            next_time: 0,
            need_sync: false,
            time_divider: 0,
            player,
            nr51: 0,
        }
    }
}

impl Sound {
    pub fn run(&mut self) {
        while self.next_time <= self.time {
            self.channel1.run(self.prev_time, self.next_time);
            self.channel2.run(self.prev_time, self.next_time);
            self.channel3.run(self.prev_time, self.next_time);
            self.channel4.run(self.prev_time, self.next_time);

            self.channel1.step_length();
            self.channel2.step_length();
            self.channel3.step_length();
            self.channel4.step_length();

            if self.time_divider == 0 {
                self.channel1.volume_envelope.step();
                self.channel2.volume_envelope.step();
                self.channel4.volume_envelope.step();
            } else if self.time_divider & 1 == 1 {
                self.channel1.step_sweep();
            }

            self.time_divider = (self.time_divider + 1) % 4;
            self.prev_time = self.next_time;
            self.next_time += CLOCKS_PER_SECOND / 256;
        }

        if self.prev_time != self.time {
            self.channel1.run(self.prev_time, self.time);
            self.channel2.run(self.prev_time, self.time);
            self.channel3.run(self.prev_time, self.time);
            self.channel4.run(self.prev_time, self.time);

            self.prev_time = self.time;
        }
    }

    pub fn tick(&mut self, clocks: u32) {
        if !self.on {
            return;
        }
        self.time += clocks;

        if self.time >= self.output_period {
            self.output();
        }
    }

    fn output(&mut self) {
        self.run();
        debug_assert!(self.time == self.prev_time);
        self.channel1.blip.end_frame(self.time);
        self.channel2.blip.end_frame(self.time);
        self.channel3.blip.end_frame(self.time);
        self.channel4.blip.end_frame(self.time);
        self.next_time -= self.time;
        self.time = 0;
        self.prev_time = 0;

        if !self.need_sync || self.player.underflowed() {
            self.need_sync = false;
            self.mix_buffers();
        } else {
            self.clear_buffers();
        }
    }

    fn mix_buffers(&mut self) {
        let sample_count = self.channel1.blip.samples_avail() as usize;
        debug_assert!(sample_count == self.channel2.blip.samples_avail() as usize);
        debug_assert!(sample_count == self.channel3.blip.samples_avail() as usize);
        debug_assert!(sample_count == self.channel4.blip.samples_avail() as usize);

        let mut outputted = 0;

        let left_vol = (f32::from(self.volume_left) / 7.0) * (1.0 / 15.0) * 0.25;
        let right_vol = (f32::from(self.volume_right) / 7.0) * (1.0 / 15.0) * 0.25;

        while outputted < sample_count {
            let buf_left = &mut [0f32; OUTPUT_SAMPLE_COUNT + 10];
            let buf_right = &mut [0f32; OUTPUT_SAMPLE_COUNT + 10];
            let buf = &mut [0i16; OUTPUT_SAMPLE_COUNT + 10];

            let count1 = self.channel1.blip.read_samples(buf, false);
            for (i, v) in buf[..count1].iter().enumerate() {
                if is_bit_on(self.nr51, 0) {
                    buf_left[i] += f32::from(*v) * left_vol;
                }
                if is_bit_on(self.nr51, 4) {
                    buf_right[i] += f32::from(*v) * right_vol;
                }
            }

            let count2 = self.channel2.blip.read_samples(buf, false);
            for (i, v) in buf[..count2].iter().enumerate() {
                if is_bit_on(self.nr51, 1) {
                    buf_left[i] += f32::from(*v) * left_vol;
                }
                if is_bit_on(self.nr51, 5) {
                    buf_right[i] += f32::from(*v) * right_vol;
                }
            }

            let count3 = self.channel3.blip.read_samples(buf, false);
            for (i, v) in buf[..count3].iter().enumerate() {
                if is_bit_on(self.nr51, 2) {
                    buf_left[i] += f32::from(*v) * left_vol;
                }
                if is_bit_on(self.nr51, 6) {
                    buf_right[i] += f32::from(*v) * right_vol;
                }
            }

            let count4 = self.channel4.blip.read_samples(buf, false);
            for (i, v) in buf[..count4].iter().enumerate() {
                if is_bit_on(self.nr51, 3) {
                    buf_left[i] += f32::from(*v) * left_vol;
                }
                if is_bit_on(self.nr51, 7) {
                    buf_right[i] += f32::from(*v) * right_vol;
                }
            }

            debug_assert!(count1 == count2);
            debug_assert!(count1 == count3);
            debug_assert!(count1 == count4);

            self.player.play(&buf_left[..count1], &buf_right[..count1]);

            outputted += count1;
        }
    }

    fn clear_buffers(&mut self) {
        self.channel1.blip.clear();
        self.channel2.blip.clear();
        self.channel3.blip.clear();
        self.channel4.blip.clear();
    }
}

impl Memory for Sound {
    fn read(&self, a: u16) -> u8 {
        match a {
            0xff10..=0xff14 => self.channel1.read(a),
            0xff16..=0xff19 => self.channel2.read(a),
            0xff1a..=0xff1e | 0xff30..=0xff3f => self.channel3.read(a),
            0xff20..=0xff23 => self.channel4.read(a),
            0xff24 => self.volume_right << 4 | self.volume_left,
            0xff25 => self.nr51,
            0xff26 => {
                (if self.on { 128 } else { 0 })
                    | (if self.channel1.on() { 1 } else { 0 })
                    | (if self.channel2.on() { 2 } else { 0 })
                    | (if self.channel3.on() { 4 } else { 0 })
                    | (if self.channel4.on() { 8 } else { 0 })
            }
            _ => 0,
        }
    }

    fn write(&mut self, a: u16, v: u8) {
        if a != 0xff26 && !self.on {
            return;
        }
        self.run();
        match a {
            0xff10..=0xff14 => self.channel1.write(a, v),
            0xff16..=0xff19 => self.channel2.write(a, v),
            0xff1a..=0xff1e | 0xff30..=0xff3f => self.channel3.write(a, v),
            0xff20..=0xff23 => self.channel4.write(a, v),
            0xff24 => {
                self.volume_left = v & 0b0111;
                self.volume_right = (v & 0b0111_0000) >> 4;
            }
            0xff25 => self.nr51 = v,
            0xff26 => self.on = is_bit_on(v, 7),
            _ => panic!(),
        }
    }
}

#[derive(Default)]
struct VolumeEnvelope {
    init_volume: u8, // 0-f
    is_amplify: bool,
    sweep_period: u8,

    delay: u8,
    volume: u8,
}

impl VolumeEnvelope {
    fn step(&mut self) {
        if self.delay > 1 {
            self.delay -= 1;
        } else if self.delay == 1 {
            self.delay = self.sweep_period;
            if self.is_amplify && self.volume < 15 {
                self.volume += 1;
            } else if !self.is_amplify && self.volume > 0 {
                self.volume -= 1;
            }
        }
    }
}

impl Memory for VolumeEnvelope {
    fn read(&self, _a: u16) -> u8 {
        0
    }

    fn write(&mut self, a: u16, v: u8) {
        match a {
            0xff12 | 0xff17 | 0xff21 => {
                self.init_volume = v >> 4;
                self.is_amplify = is_bit_on(v, 3);
                self.sweep_period = v & 0b0111;
                self.volume = self.init_volume;
            }
            0xff14 | 0xff19 | 0xff23 if is_bit_on(v, 7) => {
                self.delay = self.sweep_period;
                self.volume = self.init_volume;
            }
            _ => (),
        }
    }
}

struct SquareSound {
    nr0: u8,
    nr1: u8,
    nr2: u8,
    nr3: u8,
    nr4: u8,
    sweep_period: u8,
    sweep_negate: bool,
    sweep_shift: u8,
    duty: u8,
    new_length: u8,
    frequency: u16,
    length: u8,
    enabled: bool,
    sweep_frequency: u16,
    length_enabled: bool,
    has_sweep: bool,
    sweep_delay: u8,
    period: u32,
    volume_envelope: VolumeEnvelope,
    last_amp: i32,
    delay: u32,
    phase: u8,
    blip: BlipBuf,
}

impl SquareSound {
    fn new(blip: BlipBuf, has_sweep: bool) -> Self {
        SquareSound {
            nr0: 0,
            nr1: 0,
            nr2: 0,
            nr3: 0,
            nr4: 0,
            sweep_period: 0,
            sweep_negate: false,
            sweep_shift: 0,
            duty: 1,
            new_length: 0,
            frequency: 0,
            length: 0,
            volume_envelope: VolumeEnvelope::default(),
            enabled: false,
            sweep_frequency: 0,
            length_enabled: false,
            has_sweep,
            sweep_delay: 0,
            period: 2048,
            last_amp: 0,
            delay: 0,
            phase: 1,
            blip,
        }
    }

    fn on(&self) -> bool {
        self.enabled
    }

    fn calc_period(&mut self) {
        if self.frequency > 2048 {
            self.period = 0;
        } else {
            self.period = (2048 - u32::from(self.frequency)) * 4;
        }
    }

    fn step_sweep(&mut self) {
        if !self.has_sweep || self.sweep_period == 0 {
            return;
        }

        if self.sweep_delay > 1 {
            self.sweep_delay -= 1;
        } else {
            self.sweep_delay = self.sweep_period;
            self.frequency = self.sweep_frequency;
            if self.frequency == 2048 {
                self.enabled = false;
            }
            self.calc_period();

            let offset = self.sweep_frequency >> self.sweep_shift;
            self.sweep_frequency = if self.sweep_negate {
                if self.sweep_frequency <= offset {
                    0
                } else {
                    self.sweep_frequency - offset
                }
            } else if self.sweep_frequency >= 2048 - offset {
                2048
            } else {
                self.sweep_frequency + offset
            };
        }
    }

    fn run(&mut self, start_time: u32, end_time: u32) {
        if !self.enabled || self.period == 0 || self.volume_envelope.volume == 0 {
            if self.last_amp != 0 {
                self.blip.add_delta(start_time, -self.last_amp);
                self.last_amp = 0;
                self.delay = 0;
            }
        } else {
            let mut time = start_time + self.delay;
            let pattern = WAVE_PATTERN[self.duty as usize];
            let vol = i32::from(self.volume_envelope.volume);

            while time < end_time {
                let amp = vol * pattern[self.phase as usize];
                if amp != self.last_amp {
                    self.blip.add_delta(time, amp - self.last_amp);
                    self.last_amp = amp;
                }
                time += self.period;
                self.phase = (self.phase + 1) % 8;
            }
            self.delay = time - end_time;
        }
    }

    fn step_length(&mut self) {
        if self.length_enabled && self.length != 0 {
            self.length -= 1;
            if self.length == 0 {
                self.enabled = false;
            }
        }
    }
}

impl Memory for SquareSound {
    fn read(&self, a: u16) -> u8 {
        match a {
            0xff10 => self.nr0,
            0xff11 | 0xff16 => self.nr1,
            0xff12 | 0xff17 => self.nr2,
            0xff13 | 0xff18 => self.nr3,
            0xff14 | 0xff19 => self.nr4,
            _ => 0,
        }
    }

    fn write(&mut self, a: u16, v: u8) {
        match a {
            0xff10 => {
                self.nr0 = v;
                self.sweep_period = (v & 0b0111_0000) >> 4;
                self.sweep_negate = is_bit_on(v, 3);
                self.sweep_shift = v & 0b0111;
            }
            0xff11 | 0xff16 => {
                self.nr1 = v;
                self.duty = v >> 6;
                self.new_length = 64 - (v & 0b0011_1111);
            }
            0xff12 | 0xff17 => {
                self.nr2 = v;
                self.volume_envelope.write(a, v);
            }
            0xff13 | 0xff18 => {
                self.nr3 = v;
                self.frequency = (self.frequency & 0x0700) | u16::from(v);
                self.length = self.new_length;
                self.calc_period();
            }
            0xff14 | 0xff19 => {
                self.nr4 = v;
                self.frequency = (self.frequency & 0x00ff) | (u16::from(v & 0b0000_0111) << 8);
                self.calc_period();
                self.length_enabled = is_bit_on(v, 6);

                if is_bit_on(v, 7) {
                    self.enabled = true;
                    self.length = self.new_length;
                    self.sweep_frequency = self.frequency;
                    if self.has_sweep && self.sweep_period > 0 && self.sweep_shift > 0 {
                        self.sweep_delay = 1;
                        self.step_sweep();
                    }
                }

                self.volume_envelope.write(a, v);
            }
            _ => {}
        }
    }
}

struct WaveSound {
    nr0: u8,
    nr1: u8,
    nr2: u8,
    nr3: u8,
    nr4: u8,
    dac_enabled: bool,
    channel_enabled: bool,
    length: u8,
    new_length: u8,
    length_enabled: bool,
    volume_code: u8,
    frequency: u16,
    current_wave: u8,
    delay: u32,
    period: u32,
    waveram: [u8; 32],
    last_amp: i32,
    blip: BlipBuf,
}

impl WaveSound {
    fn new(blip: BlipBuf) -> Self {
        WaveSound {
            nr0: 0,
            nr1: 0,
            nr2: 0,
            nr3: 0,
            nr4: 0,
            dac_enabled: false,
            channel_enabled: false,
            length: 0,
            new_length: 0,
            length_enabled: false,
            volume_code: 0,
            frequency: 0,
            current_wave: 0,
            delay: 0,
            period: 0,
            waveram: [0; 32],
            last_amp: 0,
            blip,
        }
    }
}

impl WaveSound {
    fn on(&self) -> bool {
        self.channel_enabled
    }

    fn calc_period(&mut self) {
        if self.frequency > 2048 {
            self.period = 0;
        } else {
            self.period = (2048 - u32::from(self.frequency)) * 2;
        }
    }

    fn step_length(&mut self) {
        if self.length_enabled && self.length != 0 {
            self.length -= 1;
            if self.length == 0 {
                self.channel_enabled = false;
            }
        }
    }

    fn run(&mut self, start_time: u32, end_time: u32) {
        if !self.channel_enabled || self.period == 0 {
            if self.last_amp != 0 {
                self.blip.add_delta(start_time, -self.last_amp);
                self.last_amp = 0;
                self.delay = 0;
            }
        } else {
            let mut time = start_time + self.delay;

            let volshift = match self.volume_code {
                0 => 4,
                1 => 0,
                2 => 1,
                3 => 2,
                _ => unreachable!(),
            };

            while time < end_time {
                let sample = self.waveram[self.current_wave as usize];

                let amp = i32::from(sample >> volshift);

                if amp != self.last_amp {
                    self.blip.add_delta(time, amp - self.last_amp);
                    self.last_amp = amp;
                }

                time += self.period;
                self.current_wave = (self.current_wave + 1) % 32;
            }

            // next time, we have to wait an additional delay timesteps
            self.delay = time - end_time;
        }
    }
}

impl Memory for WaveSound {
    fn read(&self, a: u16) -> u8 {
        match a {
            0xff1a => self.nr0,
            0xff1b => self.nr1,
            0xff1c => self.nr2,
            0xff1d => self.nr3,
            0xff1e => self.nr4,
            0xff30..=0xff3f => {
                (self.waveram[usize::from(a - 0xff30) / 2] << 4)
                    | self.waveram[usize::from(a - 0xff30) / 2 + 1]
            }
            _ => 0,
        }
    }

    fn write(&mut self, a: u16, v: u8) {
        match a {
            0xff1a => {
                self.nr0 = v;
                self.dac_enabled = is_bit_on(v, 7);
                self.channel_enabled &= self.dac_enabled;
            }
            0xff1b => {
                self.nr1 = v;
                self.new_length = v;
            }
            0xff1c => {
                self.nr2 = v;
                self.volume_code = (v & 0b0110_0000) >> 5;
            }
            0xff1d => {
                self.nr3 = v;
                self.frequency = (self.frequency & 0b0111_0000) | u16::from(v);
                self.calc_period();
            }
            0xff1e => {
                self.nr4 = v;
                self.frequency = (self.frequency & 0x00ff) | (u16::from(v & 0b0111) << 8);
                self.calc_period();
                self.length_enabled = is_bit_on(v, 6);

                if is_bit_on(v, 7) && self.dac_enabled {
                    self.length = self.new_length;
                    self.channel_enabled = true;
                    self.current_wave = 0;
                    self.delay = 0;
                }
            }
            0xff30..=0xff3f => {
                self.waveram[usize::from(a - 0xff30) / 2] = v >> 4;
                self.waveram[usize::from(a - 0xff30) / 2 + 1] = v & 0x0f;
            }
            _ => {}
        }
    }
}

struct NoiseSound {
    nr0: u8,
    nr1: u8,
    nr2: u8,
    nr3: u8,
    nr4: u8,
    enabled: bool,
    length: u8,
    new_length: u8,
    length_enabled: bool,
    volume_envelope: VolumeEnvelope,
    period: u32,
    delay: u32,
    shift_width: u8,
    state: u16,
    last_amp: i32,
    blip: BlipBuf,
}

impl NoiseSound {
    fn new(blip: BlipBuf) -> Self {
        NoiseSound {
            nr0: 0,
            nr1: 0,
            nr2: 0,
            nr3: 0,
            nr4: 0,
            enabled: false,
            length: 0,
            new_length: 0,
            length_enabled: false,
            volume_envelope: VolumeEnvelope::default(),
            period: 2048,
            delay: 0,
            shift_width: 14,
            state: 1,
            last_amp: 0,
            blip,
        }
    }

    fn on(&self) -> bool {
        self.enabled
    }

    fn step_length(&mut self) {
        if self.length_enabled && self.length != 0 {
            self.length -= 1;
            if self.length == 0 {
                self.enabled = false;
            }
        }
    }

    fn run(&mut self, start_time: u32, end_time: u32) {
        if !self.enabled || self.volume_envelope.volume == 0 {
            if self.last_amp != 0 {
                self.blip.add_delta(start_time, -self.last_amp);
                self.last_amp = 0;
                self.delay = 0;
            }
        } else {
            let mut time = start_time + self.delay;
            while time < end_time {
                let oldstate = self.state;
                self.state <<= 1;
                let bit = ((oldstate >> self.shift_width) ^ (self.state >> self.shift_width)) & 1;
                self.state |= bit;

                let amp = match (oldstate >> self.shift_width) & 1 {
                    0 => -i32::from(self.volume_envelope.volume),
                    _ => i32::from(self.volume_envelope.volume),
                };

                if self.last_amp != amp {
                    self.blip.add_delta(time, amp - self.last_amp);
                    self.last_amp = amp;
                }

                time += self.period;
            }
            self.delay = time - end_time;
        }
    }
}

impl Memory for NoiseSound {
    fn read(&self, a: u16) -> u8 {
        match a {
            0xff1f => self.nr0,
            0xff20 => self.nr1,
            0xff21 => self.nr2,
            0xff22 => self.nr3,
            0xff23 => self.nr4,
            _ => 0,
        }
    }

    fn write(&mut self, a: u16, v: u8) {
        match a {
            0xff1f => {
                self.nr0 = v;
            }
            0xff20 => {
                self.nr1 = v;
                self.new_length = 64 - (v & 0b0011_1111);
            }
            0xff21 => {
                self.nr2 = v;
                self.volume_envelope.write(a, v);
            }
            0xff22 => {
                self.nr3 = v;
                self.shift_width = if is_bit_on(v, 3) { 6 } else { 14 };
                let freq_div = match v & 0b0111 {
                    0 => 8,
                    n => (u32::from(n) + 1) * 16,
                };
                self.period = freq_div << (v >> 4);
            }
            0xff23 => {
                self.nr4 = v;
                self.length_enabled = is_bit_on(v, 6);

                if is_bit_on(v, 7) {
                    self.enabled = true;
                    self.length = self.new_length;
                    self.state = 0xff;
                    self.delay = 0;
                }
                self.volume_envelope.write(a, v);
            }
            _ => {}
        }
    }
}
