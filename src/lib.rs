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

struct PixelBuffer(Vec<u8>);

impl PixelBuffer {
    fn new() -> Self {
        PixelBuffer(vec![0; 4 * NES_HEIGHT * NES_WIDTH])
    }

    fn render_to(&mut self, ctx: &CanvasRenderingContext2d) -> Result<(), JsValue> {
        let data = ImageData::new_with_u8_clamped_array_and_sh(
            Clamped(&mut self.0),
            NES_WIDTH as u32,
            NES_HEIGHT as u32,
        )?;
        ctx.put_image_data(&data, 0.0, 0.0)
    }
}

impl ludus::VideoDevice for PixelBuffer {
    fn blit_pixels(&mut self, pixels: &ludus::PixelBuffer) {
        for (i, pixel) in pixels.as_ref().iter().enumerate() {
            let base = 4 * i;
            self.0[base + 3] = (pixel >> 24) as u8;
            self.0[base] = (pixel >> 16) as u8;
            self.0[base + 1] = (pixel >> 8) as u8;
            self.0[base + 2] = *pixel as u8;
        }
    }
}

const SAMPLE_CHUNK_SIZE: usize = 2048;

struct Audio {
    ctx: AudioContext,
    samples: Vec<f32>,
    sample_rate: u32,
    play_timestamp: f64
}

impl Audio {
    fn new(sample_rate: u32, ctx: AudioContext) -> Self {
        Audio {
            ctx,
            samples: Vec::with_capacity(SAMPLE_CHUNK_SIZE),
            sample_rate,
            play_timestamp: 0.0
        }
    }

    #[inline]
    fn push_sample_js(&mut self, sample: f32) -> Result<(), JsValue> {
        self.samples.push(sample);
        if self.samples.len() < SAMPLE_CHUNK_SIZE {
            return Ok(())
        }
        let sample_count = self.samples.len();
        let audio_buffer = self.ctx.create_buffer(1, sample_count as u32, self.sample_rate as f32)?;
        audio_buffer.copy_to_channel(&self.samples, 0)?;
        let node = self.ctx.create_buffer_source()?;
        node.set_buffer(Some(&audio_buffer));
        node.connect_with_audio_node(&self.ctx.destination())?;
        let latency = 0.032;
        let play_timestamp = f64::max(self.ctx.current_time() + latency, self.play_timestamp);
        node.start_with_when(play_timestamp)?;
        self.play_timestamp = play_timestamp + (sample_count as f64) / (self.sample_rate as f64);
        self.samples.clear();
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

    #[wasm_bindgen]
    pub fn step(
        &mut self,
        ctx: &CanvasRenderingContext2d,
        micros: u32,
    ) -> Result<Vec<f32>, JsValue> {
        if let Some(console) = &mut self.console {
            console.step_micros(&mut self.audio, &mut self.pixels, micros);
        }
        self.pixels.render_to(ctx)?;
        Ok(Vec::new())
    }
}
