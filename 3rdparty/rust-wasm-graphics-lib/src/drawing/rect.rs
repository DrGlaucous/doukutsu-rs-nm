use wasm_bindgen::prelude::*;

use crate::canvas::Canvas;
use crate::types::ARGBColour;
use super::lines::{h_line, v_line};

#[wasm_bindgen]
/// Draws a filled rectangle of a given colour to a [`Canvas`]
///
/// [`Canvas`]: ../../canvas/struct.Canvas.html
///
/// # Arguments:
///
///   - `c`: target [`Canvas`]
///   - `col`: colour to use for fill
///   - `x1`: x co-ordinate of top-left corner
///   - `y1`: y co-ordinate of top-left corner
///   - `x2`: x co-ordinate of bottom-right corner
///   - `y2`: y co-ordinate of bottom-right corner
///
/// # Example:
///
/// ```
/// use rust_wasm_graphics_lib::canvas::Canvas;
/// use rust_wasm_graphics_lib::drawing::rect::fill_rect;
/// use rust_wasm_graphics_lib::types::ARGBColour;
///
/// let mut c = Canvas::new(16, 16);
/// fill_rect(&mut c, &ARGBColour::new(255, 255, 0, 0), 4, 4, 12, 12);
/// ```
pub fn fill_rect(c: &mut Canvas, col: &ARGBColour, mut x1: isize, mut y1: isize, mut x2: isize, mut y2: isize) {
    if x1 > x2 {
        std::mem::swap(&mut x1, &mut x2);
    }
    if y1 > y2 {
        std::mem::swap(&mut y1, &mut y2);
    }
    if x1 < 0 { x1 = 0 }
    if x2 < 0 { return; }
    if y1 < 0 { y1 = 0 }
    if y2 < 0 { return; }
    let mut x1 = x1 as usize; let mut x2 = x2 as usize;
    let mut y1 = y1 as usize; let mut y2 = y2 as usize;
    if x1 >= c.width() { x1 = c.width()  - 1 }
    if y1 >= c.height() { y1 = c.height()  - 1 }
    if x2 >= c.width() { x2 = c.width()  - 1 }
    if y2 >= c.height() { y2 = c.height()  - 1 }

    let col: u32 = col.into();
    let w = c.width();

    // Split buffer into chunks of "width", skip until the top of the rectangle and iterate over
    // all rows (y2 - y1) to get each scanline.  Then, set pixels in the scanline in the interval
    // [x1,x2].
    c.buffer_mut()
        .as_mut_slice()
        .chunks_mut(w)
        .skip(y1)
        .take(y2 - y1 + 1)
        .for_each(|scanline|
            scanline.iter_mut()
                .skip(x1)
                .take(x2 - x1 + 1)
                .for_each(|x| *x = col)
        );
}

#[wasm_bindgen]
/// Draws an un-filled rectangle with a given stroke colour to a [`Canvas`]
///
/// [`Canvas`]: ../../canvas/struct.Canvas.html
///
/// # Arguments:
///
///   - `c`: target [`Canvas`]
///   - `col`: colour to use for fill
///   - `x1`: x co-ordinate of top-left corner
///   - `y1`: y co-ordinate of top-left corner
///   - `x2`: x co-ordinate of bottom-right corner
///   - `y2`: y co-ordinate of bottom-right corner
///
/// # Example:
///
/// ```
/// use rust_wasm_graphics_lib::canvas::Canvas;
/// use rust_wasm_graphics_lib::drawing::rect::rect;
/// use rust_wasm_graphics_lib::types::ARGBColour;
///
/// let mut c = Canvas::new(16, 16);
/// rect(&mut c, &ARGBColour::new(255, 255, 0, 0), 4, 4, 12, 12);
/// ```
pub fn rect(c: &mut Canvas, col: &ARGBColour, x1: isize, y1: isize, x2: isize, y2: isize) {
    h_line(c, col, x1, y1, x2);
    h_line(c, col, x1, y2, x2);
    v_line(c, col, x1, y1, y2);
    v_line(c, col, x2, y1, y2);
}
