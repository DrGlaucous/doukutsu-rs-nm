use wasm_bindgen::prelude::*;

use crate::types::{ARGBColour, UVWrapMode};

#[wasm_bindgen]
/// A single buffer of 32-bit ARGB pixels with a fixed width and height
pub struct Canvas {
    height: usize,
    width:  usize,

    buffer: Vec<u32>,
}

#[wasm_bindgen]
impl Canvas {

    /// Create a new Canvas with a specified width and height in pixels
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            height,
            width,
            buffer: vec![0; width * height],
        }
    }

    /// Returns the index into the pixel buffer for a pixel given its x and y co-ordinates
    pub fn buffer_index(&self, x: usize, y: usize) -> usize {
        (y * self.width) + x
    }

    /// Returns a pointer to the Canvas pixel data
    pub fn buffer_ptr(&self) -> *const u32 {
        self.buffer.as_ptr()
    }

    /// Returns the Canvas's height in pixels
    pub fn height(&self) -> usize {
        self.height
    }

    /// Returns the Canvas's width in pixels
    pub fn width(&self) -> usize {
        self.width
    }

    /// Clears the entire Canvas to a specified colour
    ///
    /// # Example:
    ///
    /// ```
    /// use rust_wasm_graphics_lib::canvas::Canvas;
    /// use rust_wasm_graphics_lib::types::ARGBColour;
    ///
    /// let mut c = Canvas::new(16, 16);
    /// c.clear(&ARGBColour::new(255, 255, 0, 0));
    /// ```
    pub fn clear(&mut self, colour: &ARGBColour) {
        let colour: u32 = colour.into();
        self.buffer.iter_mut().for_each(|x| *x = colour);
    }

    /// Copies the contents of one Canvas to another starting at a specified top-left co-ordinate
    ///
    /// # Arguments:
    ///
    ///   - `src_canv`: Canvas whose contents will be copied
    ///   - `x`: X co-ordinate of starting top-left position in destination Canvas
    ///   - `y`: Y co-ordinate of starting top-left position in destination Canvas
    ///
    /// # Example:
    ///
    /// ```
    /// use rust_wasm_graphics_lib::canvas::Canvas;
    ///
    /// let c_src = Canvas::new(16, 16);
    /// let mut c_dst = Canvas::new(16, 16);
    ///
    /// // Draw c_dst to c_src at offset (4, 8)
    /// c_dst.draw_canvas(&c_src, 4, 8);
    /// ```
    pub fn draw_canvas(&mut self, src_canv: &Canvas, x: usize, y: usize) {
        if x >= self.width || y >= self.height {
            return
        }

        let max_x = if x + src_canv.width()  > self.width  { self.width  - 1 } else { x + src_canv.width()  - 1 };
        let max_y = if y + src_canv.height() > self.height { self.height - 1 } else { y + src_canv.height() - 1 };

        self.buffer
            .as_mut_slice()
            .chunks_mut(self.width)
            .skip(y)
            .take(max_y - y + 1)
            .enumerate()
            .for_each(|(dy, scanline)| {
                let src_idx = src_canv.buffer_index(0, dy);
                scanline
                    .iter_mut()
                    .skip(x)
                    .take(max_x - x + 1)
                    .enumerate()
                    .for_each(|(dx, px)| {
                        let src_px = src_canv.buffer()[src_idx + dx];
                        if src_px >> 24 > 0 {
                            *px = src_px;
                        }
                    });
            });
    }

    /// Copies a source Vec<u32> into the Canvas `buffer`
    ///
    /// The `src` vector must be the same length as the current Canvas' `width * height`.
    ///
    /// # Arguments:
    ///
    ///   - `src`: vector containing source pixel data
    ///
    /// # Example:
    ///
    /// ```
    /// use rust_wasm_graphics_lib::canvas::Canvas;
    ///
    /// let mut canv = Canvas::new(2, 2);
    /// canv.load_pixels(vec![1, 2, 3, 4]);
    /// ```
    pub fn load_pixels(&mut self, src: Vec<u32>) -> bool {
        if src.len() != self.width * self.height {
            return false;
        };
        self.buffer.as_mut_slice().copy_from_slice(src.as_slice());
        true
    }

    /// Samples the canvas buffer at a given (u,v) co-ordinate in the range [0,1] and returns the
    /// nearest pixel value.
    ///
    /// # Arguments:
    ///
    ///   - `u`: normalised U co-ordinate
    ///   - `v`: normalised V co-ordinate
    ///   - `mode`: U,V wrapping mode to UVs out of the [1,0] range
    ///
    /// # Example:
    ///
    /// ```
    /// use rust_wasm_graphics_lib::canvas::Canvas;
    /// use rust_wasm_graphics_lib::types::UVWrapMode;
    ///
    /// let mut canv = Canvas::new(8, 16);
    ///
    /// // Should return value of pixel near (4, 4)
    /// canv.sample(0.5, 0.25, UVWrapMode::Clamp);
    /// ```
    pub fn sample(&self, mut u: f64, mut v: f64, mode: UVWrapMode) -> u32 {
        match mode {
            UVWrapMode::Clamp => {
                // Clamp U,V at boundaries
                if u < 0.0 { u = 0f64; }
                if v < 0.0 { v = 0f64; }
                if u > 1.0 { u = 1f64; }
                if v > 1.0 { v = 1f64; }
            },
            UVWrapMode::Wrap => {
                // Wrap U,V at boundaries
                if u < 0.0 { u = 1.0 - ((-u) - (-u as isize) as f64); }
                if v < 0.0 { v = 1.0 - ((-v) - (-v as isize) as f64); }
                if u > 1.0 { u -= (u as isize) as f64; }
                if v > 1.0 { v -= (v as isize) as f64; }
            },
        }

        // floor() is faster than round() in benchmarks, so add 0.5 and floor() instead of round()
        let x = ((u * (self.width  - 1) as f64) + 0.5).floor() as usize;
        let y = ((v * (self.height - 1) as f64) + 0.5).floor() as usize;
        let idx = self.buffer_index(x, y);
        self.buffer[idx]
    }
}


impl Canvas {

    /// Returns an immutable reference to the pixel buffer for direct read-only pixel access
    pub fn buffer(&self) -> &Vec<u32> {
        &self.buffer
    }

    /// Returns a mutable reference to the pixel buffer for direct read/write pixel access
    pub fn buffer_mut(&mut self) -> &mut Vec<u32> {
        &mut self.buffer
    }
}
