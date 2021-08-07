use wasm_bindgen::prelude::*;
use wasm_bindgen::Clamped;
use web_sys::{CanvasRenderingContext2d, ImageData};

#[wasm_bindgen]
struct Console {
    elapsed_micros: u32,
    width: usize,
    height: usize,
    pixels: Vec<u8>,
}

#[wasm_bindgen]
impl Console {
    #[wasm_bindgen(constructor)]
    pub fn new(width: u32, height: u32) -> Self {
        Console {
            elapsed_micros: 0,
            width: width as usize,
            height: height as usize,
            pixels: vec![0; (width * height * 4) as usize],
        }
    }

    #[wasm_bindgen]
    pub fn draw(&mut self, ctx: &CanvasRenderingContext2d, micros: u32) -> Result<(), JsValue> {
        self.elapsed_micros += micros;

        for y in 0..self.height {
            for x in 0..self.width {
                let start = 4 * (y * self.width + x);
                self.pixels[start] = (100 * self.elapsed_micros / 1_000_000) as u8;
                self.pixels[start + 1] = x as u8;
                self.pixels[start + 2] = y as u8;
                self.pixels[start + 3] = 0xFF;
            }
        }
        let data = ImageData::new_with_u8_clamped_array_and_sh(
            Clamped(&mut self.pixels),
            self.width as u32,
            self.height as u32,
        )?;
        ctx.put_image_data(&data, 0.0, 0.0)
    }
}
