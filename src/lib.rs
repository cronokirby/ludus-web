use ludus;
use ludus::NES_HEIGHT;
use ludus::NES_WIDTH;
use wasm_bindgen::prelude::*;
use wasm_bindgen::Clamped;
use web_sys::AudioContext;
use web_sys::{CanvasRenderingContext2d, ImageData};

#[wasm_bindgen]
pub fn height() -> u32 {
    ludus::NES_HEIGHT as u32
}

#[wasm_bindgen]
pub fn width() -> u32 {
    ludus::NES_WIDTH as u32
}

struct NullAudioDevice;

impl ludus::AudioDevice for NullAudioDevice {
    fn push_sample(&mut self, _sample: f32) {}
}

struct PixelBuffer {
    frame_count: u32,
    buf: Vec<u8>,
}

impl PixelBuffer {
    fn new() -> Self {
        PixelBuffer {
            frame_count: 0,
            buf: vec![0; 4 * NES_HEIGHT * NES_WIDTH],
        }
    }

    fn render_to(&mut self, ctx: &CanvasRenderingContext2d) -> Result<(), JsValue> {
        let data = ImageData::new_with_u8_clamped_array_and_sh(
            Clamped(&mut self.buf),
            NES_WIDTH as u32,
            NES_HEIGHT as u32,
        )?;
        ctx.put_image_data(&data, 0.0, 0.0)
    }
}

impl ludus::VideoDevice for PixelBuffer {
    fn blit_pixels(&mut self, pixels: &ludus::PixelBuffer) {
        self.frame_count += 1;
        for (i, pixel) in pixels.as_ref().iter().enumerate() {
            let base = 4 * i;
            self.buf[base + 3] = (pixel >> 24) as u8;
            self.buf[base] = (pixel >> 16) as u8;
            self.buf[base + 1] = (pixel >> 8) as u8;
            self.buf[base + 2] = *pixel as u8;
        }
    }
}

const SAMPLE_CHUNK_SIZE: usize = 2048;

struct Audio {
    ctx: AudioContext,
    samples: Vec<f32>,
    sample_rate: u32,
    play_timestamp: f64,
    underrun: u32
}

impl Audio {
    fn new(sample_rate: u32, ctx: AudioContext) -> Self {
        Audio {
            ctx,
            samples: Vec::with_capacity(SAMPLE_CHUNK_SIZE),
            sample_rate,
            play_timestamp: 0.0,
            underrun: 0
        }
    }

    #[inline]
    fn has_chunk(&self) -> bool {
        self.samples.len() >= SAMPLE_CHUNK_SIZE
    }

    // Kudos to https://github.com/koute/pinky/blob/master/pinky-web/src/main.rs for
    // the idea behind this buffer management.
    #[inline]
    fn push_sample_js(&mut self, sample: f32) -> Result<(), JsValue> {
        self.samples.push(sample);
        if !self.has_chunk() {
            return Ok(());
        }
        let sample_count = self.samples.len();
        let audio_buffer =
            self.ctx
                .create_buffer(1, sample_count as u32, self.sample_rate as f32)?;
        audio_buffer.copy_to_channel(&self.samples, 0)?;
        let node = self.ctx.create_buffer_source()?;
        node.set_buffer(Some(&audio_buffer));
        node.connect_with_audio_node(&self.ctx.destination())?;
        let latency = 2.0 / 60.0;
        let buffered = self.play_timestamp - (self.ctx.current_time() + latency);
        let play_timestamp = f64::max(self.ctx.current_time() + latency, self.play_timestamp);
        node.start_with_when(play_timestamp)?;
        self.play_timestamp = play_timestamp + (sample_count as f64) / (self.sample_rate as f64);
        self.samples.clear();
        if buffered < 0.0 {
            self.underrun = u32::max(self.underrun, 3)
        } else if buffered < 0.01 {
            self.underrun = u32::max(self.underrun, 2)
        } else if buffered < 0.02 {
            self.underrun = u32::max(self.underrun, 1)
        }
        Ok(())
    }
}

impl ludus::AudioDevice for Audio {
    fn push_sample(&mut self, sample: f32) {
        self.push_sample_js(sample).unwrap()
    }
}

#[wasm_bindgen]
struct Emulator {
    pixels: PixelBuffer,
    audio: Audio,
    console: Option<ludus::Console>,
}

#[wasm_bindgen]
impl Emulator {
    #[wasm_bindgen(constructor)]
    pub fn new(sample_rate: u32, audio: AudioContext) -> Self {
        Emulator {
            pixels: PixelBuffer::new(),
            audio: Audio::new(sample_rate, audio),
            console: None,
        }
    }

    #[wasm_bindgen]
    pub fn update_buttons(&mut self, state: u8) {
        if let Some(console) = &mut self.console {
            console.update_controller(ludus::ButtonState {
                a: state & 1 != 0,
                b: (state >> 1) & 1 != 0,
                start: (state >> 2) & 1 != 0,
                select: (state >> 3) & 1 != 0,
                up: (state >> 4) & 1 != 0,
                down: (state >> 5) & 1 != 0,
                left: (state >> 6) & 1 != 0,
                right: (state >> 7) & 1 != 0,
            })
        }
    }

    #[wasm_bindgen]
    pub fn swap_cart(&mut self, rom: &[u8]) {
        let cart = ludus::Cart::from_bytes(rom).unwrap();
        self.console = Some(ludus::Console::new(cart, self.audio.sample_rate));
    }

    fn step_until_audio_chunk_or_frame(&mut self) {
        if let Some(console) = &mut self.console {
            let start_frame = self.pixels.frame_count;
            while (start_frame >= self.pixels.frame_count) && !self.audio.has_chunk() {
                console.step(&mut self.audio, &mut self.pixels);
            }
        }
    }

    #[wasm_bindgen]
    pub fn step(
        &mut self,
        ctx: &CanvasRenderingContext2d,
        micros: u32,
    ) -> Result<(), JsValue> {
        if let Some(console) = &mut self.console {
            console.step_micros(&mut self.audio, &mut self.pixels, micros);
        }
        self.pixels.render_to(ctx)?;
        for _ in 0..self.audio.underrun {
            self.step_until_audio_chunk_or_frame();
        }
        self.audio.underrun = 0;
        Ok(())
    }
}
