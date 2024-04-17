use wasm_bindgen::prelude::*;

use crate::canvas::Canvas;
use crate::types::ARGBColour;
use super::line_utils::{plot_line_high, plot_line_low};

#[wasm_bindgen]
/// Draws a horizontal line of a given colour to a [`Canvas`]
///
/// [`Canvas`]: ../../canvas/struct.Canvas.html
///
/// # Arguments:
///
///   - `c`: target [`Canvas`]
///   - `col`: colour to use for line
///   - `x1`: x co-ordinate of line start
///   - `y`: y co-ordinate
///   - `x2`: x co-ordinate of line end
///
/// # Example:
///
/// ```
/// use rust_wasm_graphics_lib::canvas::Canvas;
/// use rust_wasm_graphics_lib::drawing::lines::h_line;
/// use rust_wasm_graphics_lib::types::ARGBColour;
///
/// let mut c = Canvas::new(16, 16);
/// h_line(&mut c, &ARGBColour::new(255, 255, 0, 0), 4, 8, 12);
/// ```
pub fn h_line(c: &mut Canvas, col: &ARGBColour, mut x1: isize, mut y: isize, mut x2: isize) {
    if x1 > x2 {
        std::mem::swap(&mut x1, &mut x2);
    }
    if x1 < 0 { x1 = 0 }
    if x2 < 0 { return; }
    if y < 0 { y = 0 }
    let x1 = x1 as usize; let mut x2 = x2 as usize; let y = y as usize;
    if x1 > c.width() {
        return;
    }
    if y >= c.height() {
        return;
    }
    if x2 >= c.width() { x2 = c.width() - 1 }
    let col: u32 = col.into();
    let w = c.width();
    c.buffer_mut()
        .as_mut_slice()
        .iter_mut()
        .skip((y * w) + x1)
        .take(x2 - x1 + 1)
        .for_each(|x| *x = col);
}

#[wasm_bindgen]
/// Draws a vertical line of a given colour to a [`Canvas`]
///
/// [`Canvas`]: ../../canvas/struct.Canvas.html
///
/// # Arguments:
///
///   - `c`: target [`Canvas`]
///   - `col`: colour to use for line
///   - `x`: x co-ordinate
///   - `y1`: y co-ordinate of line start
///   - `y2`: y co-ordinate of line end
///
/// # Example:
///
/// ```
/// use rust_wasm_graphics_lib::canvas::Canvas;
/// use rust_wasm_graphics_lib::drawing::lines::v_line;
/// use rust_wasm_graphics_lib::types::ARGBColour;
///
/// let mut c = Canvas::new(16, 16);
/// v_line(&mut c, &ARGBColour::new(255, 255, 0, 0), 8, 4, 12);
/// ```
pub fn v_line(c: &mut Canvas, col: &ARGBColour, mut x: isize, mut y1: isize, mut y2: isize) {
    if y1 > y2 {
        std::mem::swap(&mut y1, &mut y2);
    }
    if x < 0 { x = 0 }
    if y1 < 0 { y1 = 0 }
    if y2 < 0 { return; }
    let x = x as usize; let y1 = y1 as usize; let mut y2 = y2 as usize;
    if x >= c.width() {
        return;
    }
    if y1 > c.height() {
        return;
    }
    if y2 >= c.height() { y2 = c.height() - 1 }
    let col: u32 = col.into();
    let w = c.width();
    let mut idx = c.buffer_index(x, y1);
    for _ in y1..=y2 {
        c.buffer_mut()[idx] = col;
        idx += w;
    }
}

/// Draws a line of a given colour to a [`Canvas`] using Bresenham's line drawing algorithm.
///
/// [`Canvas`]: ../../canvas/struct.Canvas.html
///
/// # Arguments:
///
///   - `c`: target [`Canvas`]
///   - `col`: colour to use for line
///   - `x1`: x co-ordinate of line start
///   - `y1`: y co-ordinate of line start
///   - `x2`: x co-ordinate of line end
///   - `y2`: y co-ordinate of line end
///
/// # Example:
///
/// ```
/// use rust_wasm_graphics_lib::canvas::Canvas;
/// use rust_wasm_graphics_lib::drawing::lines::line_bresenham;
/// use rust_wasm_graphics_lib::types::ARGBColour;
///
/// let mut c = Canvas::new(128, 128);
/// line_bresenham(&mut c, &ARGBColour::new(255, 255, 0, 0), 10, 20, 110, 120);
/// ```
pub fn line_bresenham(c: &mut Canvas, col: &ARGBColour, mut x1: isize, mut y1: isize, x2: isize, y2: isize) {
    if x1 < 0 { x1 = 0 }
    if x2 < 0 { return; }
    if y1 < 0 { y1 = 0 }
    if y2 < 0 { return; }
    let mut x1 = x1 as usize; let mut x2 = x2 as usize;
    let mut y1 = y1 as usize; let mut y2 = y2 as usize;
    if x2 > x1 && x1 > c.width() { return; }
    if y2 > y1 && y1 > c.height() { return; }
    if x1 > x2 && x2 > c.width() { return; }
    if y1 > y2 && y2 > c.height() { return; }
    if x1 > x2 && x1 >= c.width() { x1 = c.width() - 1 }
    if y1 > y2 && y1 >= c.height() { y1 = c.height() - 1 }
    if x2 > x1 && x2 >= c.width() { x2 = c.width() - 1 }
    if y2 > y1 && y2 >= c.height() { y2 = c.height() - 1 }

    let col: u32 = col.into();

    if ((y2 as isize) - (y1 as isize)).abs() < ((x2 as isize) - (x1 as isize)).abs() {
        if x1 > x2 {
            plot_line_low(c, col, x2, y2, x1, y1);
        } else {
            plot_line_low(c, col, x1, y1, x2, y2);
        }
    } else if y1 > y2 {
        plot_line_high(c, col, x2, y2, x1, y1);
    } else {
        plot_line_high(c, col, x1, y1, x2, y2);
    }
}

#[wasm_bindgen]
/// General-purpose line drawing function.  Draws a line of a given colour between points (x1,x2)
/// and (y1,y2) to a [`Canvas`].
///
/// Defers actual drawing to [`h_line`], [`v_line`] or [`line_bresenham`] as appropriate.
///
/// [`Canvas`]: ../../canvas/struct.Canvas.html
/// [`h_line`]: ./fn.h_line.html
/// [`v_line`]: ./fn.v_line.html
/// [`line_bresenham`]: ./fn.line_bresenham.html
///
/// # Arguments:
///
///   - `c`: target [`Canvas`]
///   - `col`: colour to use for line
///   - `x1`: x co-ordinate of line start
///   - `y1`: y co-ordinate of line start
///   - `x2`: x co-ordinate of line end
///   - `y2`: y co-ordinate of line end
///
/// # Example:
///
/// ```
/// use rust_wasm_graphics_lib::canvas::Canvas;
/// use rust_wasm_graphics_lib::drawing::lines::line;
/// use rust_wasm_graphics_lib::types::ARGBColour;
///
/// let mut c = Canvas::new(128, 128);
/// line(&mut c, &ARGBColour::new(255, 255, 0, 0), 10, 20, 110, 120);
/// ```
pub fn line(c: &mut Canvas, col: &ARGBColour, x1: isize, y1: isize, x2: isize, y2: isize) {
    if x1 == x2 {
        v_line(c, col, x1, y1, y2);
    } else if y1 == y2 {
        h_line(c, col, x1, y1, x2);
    } else {
        line_bresenham(c, col, x1, y1, x2, y2);
    }
}
