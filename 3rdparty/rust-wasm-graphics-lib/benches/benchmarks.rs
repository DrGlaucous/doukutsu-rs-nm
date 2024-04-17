use criterion::{Criterion, criterion_group, criterion_main};

use rust_wasm_graphics_lib::canvas::Canvas;
use rust_wasm_graphics_lib::drawing::shape::{
    fill_polygon,
    fill_triangle,
    polygon,
    textured_triangle,
};
use rust_wasm_graphics_lib::drawing::rect::fill_rect;
use rust_wasm_graphics_lib::drawing::lines::{
    h_line,
    v_line,
};
use rust_wasm_graphics_lib::types::{
    ARGBColour,
    UVWrapMode,
    UVVertex,
};

fn bench_canvas_clear(c: &mut Criterion) {
    let col = ARGBColour::new(255, 255, 0, 0);
    let mut can = Canvas::new(128, 128);
    c.bench_function("Canvas::clear()", move |b| b.iter(|| can.clear(&col)));
}

fn bench_canvas_draw_canvas(c: &mut Criterion) {
    let src = Canvas::new(128, 128);
    let mut dst = Canvas::new(128, 128);
    c.bench_function("Canvas::draw_canvas()", move |b| b.iter(|| dst.draw_canvas(&src, 0, 0)));
}

fn bench_canvas_load_pixels(c: &mut Criterion) {
    let mut dst = Canvas::new(128, 128);
    let pixels = vec![0; 128 * 128];
    c.bench_function("Canvas::load_pixels()", move |b| b.iter(|| dst.load_pixels(pixels.clone())));
}

fn bench_canvas_sample(c: &mut Criterion) {
    let can = Canvas::new(128, 128);
    c.bench_function("Canvas::sample()", move |b| b.iter(|| can.sample(0.22, 0.77, UVWrapMode::Wrap)));
}


fn bench_drawing_fill_polygon(c: &mut Criterion) {
    let col = ARGBColour::new(255, 255, 0, 0);
    let mut can = Canvas::new(128, 128);
    let pts = vec![10, 10, 60, 60, 20, 110, 10, 10];
    c.bench_function(
        "drawing::fill_polygon()",
        move |b| b.iter(|| fill_polygon(&mut can, &col, pts.clone()))
    );
}

fn bench_drawing_fill_rect(c: &mut Criterion) {
    let col = ARGBColour::new(255, 255, 0, 0);
    let mut can = Canvas::new(128, 128);
    c.bench_function(
        "drawing::fill_rect()",
        move |b| b.iter(|| fill_rect(&mut can, &col, 16, 16, 112, 112))
    );
}

fn bench_drawing_fill_triangle(c: &mut Criterion) {
    let col = ARGBColour::new(255, 255, 0, 0);
    let mut can = Canvas::new(128, 128);
    c.bench_function(
        "drawing::fill_triangle()",
        move |b| b.iter(|| fill_triangle(&mut can, &col, 10, 10, 60, 60, 20, 110))
    );
}

fn bench_drawing_h_line(c: &mut Criterion) {
    let col = ARGBColour::new(255, 255, 0, 0);
    let mut can = Canvas::new(128, 128);
    c.bench_function(
        "drawing::h_line()",
        move |b| b.iter(|| h_line(&mut can, &col, 16, 64, 112))
    );
}

fn bench_drawing_polygon(c: &mut Criterion) {
    let col = ARGBColour::new(255, 255, 0, 0);
    let mut can = Canvas::new(128, 128);
    let pts = vec![0, -10, 10, 10, -10, 10];
    c.bench_function(
        "drawing::polygon()",
        move |b| b.iter(|| polygon(&mut can, &col, true, pts.clone()))
    );
}

fn bench_drawing_textured_triangle(c: &mut Criterion) {
    let mut c_dst = Canvas::new(128, 128);
    let c_src = Canvas::new(64, 64);
    let pta = UVVertex::new(16, 16, 0.0, 0.0);
    let ptb = UVVertex::new(112, 96, 0.5, 1.0);
    let ptc = UVVertex::new(32, 128, 1.0, 0.75);
    c.bench_function(
        "drawing::textured_triangle()",
        move |b| b.iter(|| {
            textured_triangle(&mut c_dst, &c_src, &pta, &ptb, &ptc, UVWrapMode::Wrap);
        }),
    );
}

fn bench_drawing_v_line(c: &mut Criterion) {
    let col = ARGBColour::new(255, 255, 0, 0);
    let mut can = Canvas::new(128, 128);
    c.bench_function(
        "drawing::v_line()",
        move |b| b.iter(|| v_line(&mut can, &col, 64, 16, 112))
    );
}


criterion_group!(benches,
    bench_canvas_clear,
    bench_canvas_draw_canvas,
    bench_canvas_load_pixels,
    bench_canvas_sample,
    bench_drawing_fill_polygon,
    bench_drawing_fill_rect,
    bench_drawing_fill_triangle,
    bench_drawing_h_line,
    bench_drawing_polygon,
    bench_drawing_textured_triangle,
    bench_drawing_v_line,
);

criterion_main!(benches);
