use super::lines::h_line;
use crate::canvas::Canvas;
use crate::types::{ARGBColour, UVWrapMode, UVVertex};

pub fn downward_triangle(
    c: &mut Canvas,
    col: &ARGBColour,
    tl_x: isize,
    tl_y: isize,
    tr_x: isize,
    tr_y: isize,
    bot_x: isize,
    bot_y: isize,
) {
    let dxl = (bot_x - tl_x) as f64 / (bot_y - tl_y) as f64;
    let dxr = (bot_x - tr_x) as f64 / (bot_y - tr_y) as f64;
    for y in tl_y..=bot_y {
        let xl = tl_x + ((y - tl_y) as f64 * dxl) as isize;
        let xr = tr_x + ((y - tl_y) as f64 * dxr) as isize;
        h_line(c, col, xl, y, xr);
    }
}

pub fn upward_triangle(
    c: &mut Canvas,
    col: &ARGBColour,
    top_x: isize,
    top_y: isize,
    bl_x: isize,
    bl_y: isize,
    br_x: isize,
    br_y: isize,
) {
    let dxl = (bl_x - top_x) as f64 / (bl_y - top_y) as f64;
    let dxr = (br_x - top_x) as f64 / (br_y - top_y) as f64;
    for y in top_y..=bl_y {
        let xl = bl_x + ((y - bl_y) as f64 * dxl) as isize;
        let xr = br_x + ((y - bl_y) as f64 * dxr) as isize;
        h_line(c, col, xl, y, xr);
    }
}

pub fn downward_triangle_textured(
    canv_dst: &mut Canvas,
    canv_src: &Canvas,
    tl: UVVertex,
    tr: UVVertex,
    bot: UVVertex,
    uv_mode: UVWrapMode,
) {
    let dxl = (bot.x - tl.x) as f64 / (bot.y - tl.y) as f64;
    let dxr = (bot.x - tr.x) as f64 / (bot.y - tr.y) as f64;
    let dul = (bot.u - tl.u) as f64 / (bot.y - tl.y) as f64;
    let dur = (bot.u - tr.u) as f64 / (bot.y - tr.y) as f64;
    let dvl = (bot.v - tl.v) as f64 / (bot.y - tl.y) as f64;
    let dvr = (bot.v - tr.v) as f64 / (bot.y - tr.y) as f64;
    let y_start = if tl.y > 0 { tl.y } else { 0 };
    let y_end = if bot.y < canv_dst.height() as isize {
        bot.y
    } else {
        canv_dst.height() as isize - 1
    };
    for y in y_start..=y_end {
        let mut xl = tl.x + ((y - tl.y) as f64 * dxl) as isize;
        let mut xr = tr.x + ((y - tl.y) as f64 * dxr) as isize;
        let ul = tl.u + ((y - tl.y) as f64 * dul);
        let ur = tr.u + ((y - tl.y) as f64 * dur);
        let vl = tl.v + ((y - tl.y) as f64 * dvl);
        let vr = tr.v + ((y - tl.y) as f64 * dvr);
        let du = (ur - ul) / (xr - xl) as f64;
        let dv = (vr - vl) / (xr - xl) as f64;
        let mut u = ul;
        let mut v = vl;

        // Clamp scanline X bounds and re-calculate U,V if necessary
        if xl < 0 {
            u += du * (-xl as f64);
            v += dv * (-xl as f64);
            xl = 0;
        }
        if xr >= canv_dst.width() as isize { xr = canv_dst.width() as isize - 1; }

        for x in xl..=xr {
            let col = canv_src.sample(u, v, uv_mode);
            let idx = canv_dst.buffer_index(x as usize, y as usize);
            if col >> 24 > 0 {
                canv_dst.buffer_mut()[idx] = col;
            }
            u += du;
            v += dv;
        }
    }
}

pub fn upward_triangle_textured(
    canv_dst: &mut Canvas,
    canv_src: &Canvas,
    top: UVVertex,
    bl: UVVertex,
    br: UVVertex,
    uv_mode: UVWrapMode,
) {
    let dxl = (bl.x - top.x) as f64 / (bl.y - top.y) as f64;
    let dxr = (br.x - top.x) as f64 / (br.y - top.y) as f64;
    let dul = (bl.u - top.u) as f64 / (bl.y - top.y) as f64;
    let dur = (br.u - top.u) as f64 / (br.y - top.y) as f64;
    let dvl = (bl.v - top.v) as f64 / (bl.y - top.y) as f64;
    let dvr = (br.v - top.v) as f64 / (br.y - top.y) as f64;
    let y_start = if top.y > 0 { top.y } else { 0 };
    let y_end = if bl.y < canv_dst.height() as isize {
        bl.y
    } else {
        canv_dst.height() as isize - 1
    };
    for y in y_start..=y_end {
        let mut xl = bl.x + ((y - bl.y) as f64 * dxl) as isize;
        let mut xr = br.x + ((y - bl.y) as f64 * dxr) as isize;
        let ul = bl.u + ((y - bl.y) as f64 * dul);
        let ur = br.u + ((y - bl.y) as f64 * dur);
        let vl = bl.v + ((y - bl.y) as f64 * dvl);
        let vr = br.v + ((y - bl.y) as f64 * dvr);
        let du = (ur - ul) / (xr - xl) as f64;
        let dv = (vr - vl) / (xr - xl) as f64;
        let mut u = ul;
        let mut v = vl;

        // Clamp scanline X bounds and re-calculate U,V if necessary
        if xl < 0 {
            u += du * (-xl as f64);
            v += dv * (-xl as f64);
            xl = 0;
        }
        if xr >= canv_dst.width() as isize { xr = canv_dst.width() as isize - 1; }

        for x in xl..=xr {
            let col = canv_src.sample(u, v, uv_mode);
            let idx = canv_dst.buffer_index(x as usize, y as usize);
            if col >> 24 > 0 {
                canv_dst.buffer_mut()[idx] = col;
            }
            u += du;
            v += dv;
        }
    }
}
