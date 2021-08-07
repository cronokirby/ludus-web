use std::ptr;

use ludus;
use ludus::NES_HEIGHT;
use ludus::NES_WIDTH;
use wasm_bindgen::prelude::*;
use wasm_bindgen::Clamped;
use web_sys::{CanvasRenderingContext2d, ImageData};

const SAMPLE_RATE: u32 = 4000;

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

#[wasm_bindgen]
struct Emulator {
    pixels: PixelBuffer,
    audio: [f32; 3],
    console: Option<ludus::Console>,
}

#[wasm_bindgen]
impl Emulator {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Emulator {
            pixels: PixelBuffer::new(),
            audio: [0.0, 0.1, 0.2],
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
        self.console = Some(ludus::Console::new(cart, SAMPLE_RATE));
    }

    #[wasm_bindgen]
    pub fn step(&mut self, ctx: &CanvasRenderingContext2d, micros: u32) -> Result<(), JsValue> {
        if let Some(console) = &mut self.console {
            console.step_micros(&mut NullAudioDevice, &mut self.pixels, micros);
        }
        self.pixels.render_to(ctx)
    }
}
