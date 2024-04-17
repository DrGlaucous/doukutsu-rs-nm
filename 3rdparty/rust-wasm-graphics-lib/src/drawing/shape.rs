use wasm_bindgen::prelude::*;

use crate::canvas::Canvas;
use crate::types::{ARGBColour, UVWrapMode, UVVertex};
use super::lines::{h_line, line, v_line};
use super::shape_utils::{
    downward_triangle,
    downward_triangle_textured,
    upward_triangle,
    upward_triangle_textured,
};

#[wasm_bindgen]
/// Draws an un-filled polygon given a list of vertices with a given stroke colour.
///
/// Vertices should be listed as a flat array of x,y co-ordinates, therefore the length should
/// always be a multiple of 2.
///
/// If the `close` flag is set, a line will also be drawn from the last vertex to the first to
/// close the polygon.
///
/// [`Canvas`]: ../../canvas/struct.Canvas.html
///
/// # Arguments:
///
///   - `c`: target [`Canvas`]
///   - `col`: colour to use for line
///   - `close`: if set, will close the polygon by drawing a line from the last vertex to the
///      first.
///   - `points`: flat list of vertices with components in x,y order
///
/// # Example:
///
/// ```
/// use rust_wasm_graphics_lib::canvas::Canvas;
/// use rust_wasm_graphics_lib::drawing::shape::polygon;
/// use rust_wasm_graphics_lib::types::ARGBColour;
///
/// let mut c = Canvas::new(128, 128);
///
/// // Draw a triangle
/// polygon(&mut c, &ARGBColour::new(255, 255, 0, 0), true, vec![0, -10, 10, 10, -10, 10]);
/// ```
pub fn polygon(c: &mut Canvas, col: &ARGBColour, close: bool, points: Vec<i32>) {
    points.as_slice().chunks(2).zip(points.as_slice().chunks(2).skip(1)).for_each(|(curr, next)| {
        line(c, col, curr[0] as isize, curr[1] as isize, next[0] as isize, next[1] as isize);
    });
    if close && points.len() >= 4 {
        let fx = points[0];
        let fy = points[1];
        let lx = points[points.len() - 2];
        let ly = points[points.len() - 1];
        line(c, col, lx as isize, ly as isize, fx as isize, fy as isize);
    }
}

#[wasm_bindgen]
/// Draws a filled polygon given a list of vertices with a given fill colour.
///
/// Note that the polygon should be closed, meaning that the last vertex should match the first, in
/// order for the routine to work correctly.
///
/// See [`polygon()`] for further details.
///
/// [`Canvas`]: ../../canvas/struct.Canvas.html
/// [`polygon()`]: ./fn.polygon.html
///
/// # Arguments:
///
///   - `c`: target [`Canvas`]
///   - `col`: colour to use for line
///   - `close`: if set, will close the polygon by drawing a line from the last vertex to the
///      first.
///   - `points`: flat list of vertices with components in x,y order
///
/// # Example:
///
/// ```
/// use rust_wasm_graphics_lib::canvas::Canvas;
/// use rust_wasm_graphics_lib::drawing::shape::fill_polygon;
/// use rust_wasm_graphics_lib::types::ARGBColour;
///
/// let mut c = Canvas::new(128, 128);
///
/// // Draw a filled triangle
/// fill_polygon(&mut c, &ARGBColour::new(255, 255, 0, 0), vec![0, -10, 10, 10, -10, 10, 0, -10]);
/// ```
pub fn fill_polygon(c: &mut Canvas, col: &ARGBColour, points: Vec<i32>) {

    // TODO: optimise!

    let ymin = points.as_slice().chunks(2).map(|x| x[1]).min().unwrap_or(0);
    let ymax = points.as_slice().chunks(2).map(|x| x[1]).max().unwrap_or(0);

    for y in ymin..=ymax {

        // Get all edges as a vector of start/end vertices (each vertex is (x, y)).
        let mut xs: Vec<i32> = points
            .as_slice()
            .chunks(2)
            .zip(points.as_slice().chunks(2).skip(1))
            .map(|(s, e)| ((s[0], s[1]), (e[0], e[1])))

            // Get x co-ordinate of each edge at y
            .map(|e| {
                let miny = if (e.1).1 < (e.0).1 { (e.1).1 } else { (e.0).1 };
                let maxy = if (e.1).1 > (e.0).1 { (e.1).1 } else { (e.0).1 };
                let sx = (e.0).0;
                let ex = (e.1).0;
                let sy = (e.0).1;
                let ey = (e.1).1;
                if miny == maxy || y < miny || y >= maxy {
                    None
                } else if y == maxy {
                    Some(ex)
                } else {
                    let dy = (y - sy) as f32 / (ey - sy) as f32;
                    Some(sx + ((ex as f32 - sx as f32) * dy) as i32)
                }
            })
            .filter(|x| x.is_some())
            .map(|x| x.unwrap())
            .collect();

        xs.sort_unstable();

        // Draw h_line between all edge x co-ordinates
        xs.as_slice().chunks(2).for_each(|x|
            if x.len() == 2 {
                h_line(c, col, x[0] as isize, y as isize, x[1] as isize)
            }
        );

        // Handle all horizontal edges on this scanline
        points
            .as_slice()
            .chunks(2)
            .zip(points.as_slice().chunks(2).skip(1))
            .map(|(s, e)| ((s[0], s[1]), (e[0], e[1])))
            .filter(|e| (e.0).1 == y && (e.0).1 == (e.1).1)
            .for_each(|e| {
                h_line(c, col, (e.0).0 as isize, y as isize, (e.1).0 as isize)
            });
    }
}

#[wasm_bindgen]
/// Draws a filled triangle with a given fill colour given three vertices.
///
/// Vertices are specified as two `isize` co-ordinates in x,y order.
///
/// _NOTE_: this routine is much faster than [`fill_polygon()`] and should be used whenever
/// possible.
///
/// [`Canvas`]: ../../canvas/struct.Canvas.html
/// [`fill_polygon()`]: ./fn.fill_polygon.html
///
/// # Arguments:
///
///   - `c`: target [`Canvas`]
///   - `col`: colour to use for line
///   - `x1`: first vertex x co-ordinate
///   - `y1`: first vertex y co-ordinate
///   - `x2`: second vertex x co-ordinate
///   - `y2`: second vertex y co-ordinate
///   - `x3`: third vertex x co-ordinate
///   - `y3`: third vertex y co-ordinate
///
/// # Example:
///
/// ```
/// use rust_wasm_graphics_lib::canvas::Canvas;
/// use rust_wasm_graphics_lib::drawing::shape::fill_triangle;
/// use rust_wasm_graphics_lib::types::ARGBColour;
///
/// let mut c = Canvas::new(128, 128);
/// fill_triangle(&mut c, &ARGBColour::new(255, 255, 0, 0), 0, -10, 10, 10, -10, 10);
/// ```
pub fn fill_triangle(
    c: &mut Canvas,
    col: &ARGBColour,
    mut x1: isize,
    mut y1: isize,
    mut x2: isize,
    mut y2: isize,
    mut x3: isize,
    mut y3: isize,
) {
    // Sort vertices in y order
    if y1 > y2 {
        std::mem::swap(&mut x1, &mut x2);
        std::mem::swap(&mut y1, &mut y2);
    }
    if y1 > y3 {
        std::mem::swap(&mut x1, &mut x3);
        std::mem::swap(&mut y1, &mut y3);
    }
    if y2 > y3 {
        std::mem::swap(&mut x2, &mut x3);
        std::mem::swap(&mut y2, &mut y3);
    }

    // Draw a single line if the triangle vertices are aligned along an axis
    if y1 == y2 && y2 == y3 {
        let pts = vec![x1, x2, x3];
        let minx = pts.iter().min().unwrap();
        let maxx = pts.iter().max().unwrap();
        h_line(c, col, *minx, y1, *maxx);
    }
    if x1 == x2 && x2 == x3 {
        let pts = vec![y1, y2, y3];
        let miny = pts.iter().min().unwrap();
        let maxy = pts.iter().max().unwrap();
        v_line(c, col, x1, *miny, *maxy);
    }

    if y1 == y2 {
        if x1 > x2 {
            std::mem::swap(&mut x1, &mut x2);
            std::mem::swap(&mut y1, &mut y2);
        }
        downward_triangle(c, col, x1, y1, x2, y2, x3, y3);
    } else if y2 == y3 {
        if x2 > x3 {
            std::mem::swap(&mut x2, &mut x3);
            std::mem::swap(&mut y2, &mut y3);
        }
        upward_triangle(c, col, x1, y1, x2, y2, x3, y3);
    } else {

        // Split triangle in two
        let mut new_pt_y = y2;
        let dy = (y2 - y1) as f64 / (y3 - y1) as f64;
        let mut new_pt_x = x1 + ((x3 - x1) as f64 * dy) as isize;

        // Make sure new point is to the right
        if x2 > new_pt_x {
            std::mem::swap(&mut x2, &mut new_pt_x);
            std::mem::swap(&mut y2, &mut new_pt_y);
        }

        // Call downward_triangle() and upward_triangle() for new split triangles
        upward_triangle(c, col, x1, y1, x2, y2, new_pt_x, new_pt_y);
        downward_triangle(c, col, x2, y2, new_pt_x, new_pt_y, x3, y3);
    }
}

#[wasm_bindgen]
/// Draws a filled triangle given three [`UVVertex`] vertices and samples pixels from the `canv_src`
/// [`Canvas`].
///
/// [`Canvas`]: ../../canvas/struct.Canvas.html
/// [`UVWrapMode`]: ../../types/enum.UVWrapMode.html
/// [`UVVertex`]: ../../types/struct.UVVertex.html
///
/// # Arguments:
///
///   - `canv_dest`: target [`Canvas`]
///   - `canv_src`: [`Canvas`] used for sampling pixels
///   - `a`: first vertex
///   - `b`: second vertex
///   - `c`: third vertex
///   - `uv_mode`: UV wrapping mode (see [`UVWrapMode`])
///
/// # Example:
///
/// ```
/// use rust_wasm_graphics_lib::canvas::Canvas;
/// use rust_wasm_graphics_lib::drawing::shape::textured_triangle;
/// use rust_wasm_graphics_lib::types::{UVWrapMode, UVVertex};
///
/// let mut c = Canvas::new(128, 128);
///
/// // Canvas containing pixels to sample as texture
/// let c2 = Canvas::new(64, 64);
///
/// // Draw triangle on "c" using pixels sampled from "c2".
/// textured_triangle(
///     &mut c,
///     &c2,
///     &UVVertex::new(  0, -10, 0.5, 0.0),
///     &UVVertex::new( 10,  10, 1.0, 1.0),
///     &UVVertex::new(-10,  10, 0.0, 1.0),
///     UVWrapMode::Wrap,
/// );
/// ```
pub fn textured_triangle(
    canv_dst: &mut Canvas,
    canv_src: &Canvas,
    a: &UVVertex,
    b: &UVVertex,
    c: &UVVertex,
    uv_mode: UVWrapMode,
) {
    let mut a = *a;
    let mut b = *b;
    let mut c = *c;

    // Sort vertices in y order
    if a.y > b.y {
        std::mem::swap(&mut a, &mut b);
    }
    if a.y > c.y {
        std::mem::swap(&mut a, &mut c);
    }
    if b.y > c.y {
        std::mem::swap(&mut b, &mut c);
    }

    if a.y == b.y && b.y == c.y {
        return;
    }
    if a.x == b.x && b.x == c.x {
        return;
    }

    if a.y == b.y {
        if a.x > b.x {
            std::mem::swap(&mut a, &mut b);
        }
        downward_triangle_textured(canv_dst, canv_src, a, b, c, uv_mode);
    } else if b.y == c.y {
        if b.x > c.x {
            std::mem::swap(&mut b, &mut c);
        }
        upward_triangle_textured(canv_dst, canv_src, a, b, c, uv_mode);
    } else {

        // Split triangle in two
        let dy = (b.y - a.y) as f64 / (c.y - a.y) as f64;
        let mut new_vert = UVVertex::new(
            a.x + ((c.x - a.x) as f64 * dy) as isize,
            b.y,

            // Calculate new_vert U,V
            a.u + ((c.u - a.u) * dy),
            a.v + ((c.v - a.v) * dy),
        );

        // Make sure new point is to the right
        if b.x > new_vert.x {
            std::mem::swap(&mut b, &mut new_vert);
        }

        // Call downward_triangle() and upward_triangle() for new split triangles
        upward_triangle_textured(canv_dst, canv_src, a, b, new_vert, uv_mode);
        downward_triangle_textured(canv_dst, canv_src, b, new_vert, c, uv_mode);
    }
}
