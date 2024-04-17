//! Test suite for the Web and headless browsers.

#![cfg(target_arch = "wasm32")]

extern crate wasm_bindgen_test;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

extern crate rust_wasm_graphics_lib;
use crate::rust_wasm_graphics_lib::canvas::{self, Canvas};
use crate::rust_wasm_graphics_lib::drawing;
use crate::rust_wasm_graphics_lib::types::{self, ARGBColour, UVWrapMode};

fn assert_no_pixels_with_colour(canv: &Canvas, col: &ARGBColour) {
    let col_u32: u32 = col.into();
    assert_eq!(canv.buffer().iter().any(|x| *x == col_u32), false);
}

fn assert_all_pixels_have_colour(canv: &Canvas, col: &ARGBColour) {
    let col_u32: u32 = col.into();
    assert_eq!(canv.buffer().iter().all(|x| *x == col_u32), true);
}

fn assert_pixels_with_colour(canv: &Canvas, col: &ARGBColour, coords: &Vec<(usize, usize)>) {
    let col_u32: u32 = col.into();
    assert_eq!(
        coords.iter().all(|x| {
            let idx = canv.buffer_index(x.0, x.1);
            canv.buffer()[idx] == col_u32
        }),
        true
    );
}

fn assert_pixels_without_colour(canv: &Canvas, col: &ARGBColour, coords: &Vec<(usize, usize)>) {
    let col_u32: u32 = col.into();
    assert_eq!(
        coords.iter().all(|x| {
            let idx = canv.buffer_index(x.0, x.1);
            canv.buffer()[idx] != col_u32
        }),
        true
    );
}


#[wasm_bindgen_test]
fn argbcolour_into_u32() {
    assert_eq!(u32::from(&ARGBColour::new(0, 0, 0, 0)), 0);
    assert_eq!(u32::from(&ARGBColour::new(255, 255, 255, 255)), 0xFFFFFFFF);
    assert_eq!(u32::from(&ARGBColour::new(1, 2, 3, 4)), 0x01020304);
}


#[wasm_bindgen_test]
fn canvas_create() {
    let canv = Canvas::new(10, 20);
    assert_eq!(canv.width(),  10);
    assert_eq!(canv.height(), 20);
    assert_eq!(canv.buffer().len(), 10 * 20);
}

#[wasm_bindgen_test]
fn canvas_buffer_index() {
    let canv = Canvas::new(10, 20);
    assert_eq!(canv.buffer_index(5, 6), 6 * 10 + 5);
    assert_eq!(canv.buffer_index(7, 8), 8 * 10 + 7);
    let canv = Canvas::new(15, 28);
    assert_eq!(canv.buffer_index(5, 6), 6 * 15 + 5);
    assert_eq!(canv.buffer_index(7, 8), 8 * 15 + 7);
}

#[wasm_bindgen_test]
fn canvas_clear() {
    let mut canv = Canvas::new(3, 4);
    let col = ARGBColour::new(255, 255, 0, 0);

    // Assert that no canvas pixels have "col" value
    assert_no_pixels_with_colour(&canv, &col);

    // Clear the canvas to "col"
    canv.clear(&col);

    // Assert that all canvas pixels now have "col" value
    assert_all_pixels_have_colour(&canv, &col);
}

#[wasm_bindgen_test]
fn canvas_draw_canvas() {
    let mut dst = Canvas::new(3, 3);
    let mut src = Canvas::new(3, 3);
    let col = ARGBColour::new(255, 255, 0, 0);
    let col_u32 = u32::from(&col);

    // Draw: -
    //  +---+
    //  |#  |
    //  | # |
    //  |# #|
    //  +---+
    src.buffer_mut()[0] = col_u32;
    src.buffer_mut()[4] = col_u32;
    src.buffer_mut()[6] = col_u32;
    src.buffer_mut()[8] = col_u32;

    assert_no_pixels_with_colour(&dst, &col);

    // Should produce: -
    //  +---+
    //  |   |
    //  | # |
    //  |  #|
    //  +---+
    dst.draw_canvas(&mut src, 1, 1);

    assert_pixels_without_colour(
        &dst,
        &col,
        &vec![
            (0, 0), (1, 0), (2, 0),
            (0, 1), /*   */ (2, 1),
            (0, 2), (1, 2), /*   */
        ],
    );
    assert_pixels_with_colour(&dst, &col, &vec![(1, 1), (2, 2)]);
}

#[wasm_bindgen_test]
fn canvas_load_pixels() {
    let mut dst = Canvas::new(3, 3);
    let col = ARGBColour::new(255, 255, 0, 0);
    let col_u32 = u32::from(&col);
    let src = vec![
        col_u32, 0,       col_u32,
        0,       col_u32, col_u32,
        0,       0,       col_u32,
    ];

    assert_no_pixels_with_colour(&dst, &col);
    assert!(dst.load_pixels(src));

    assert_pixels_without_colour(
        &dst,
        &col,
        &vec![
            /*   */ (1, 0), /*   */
            (0, 1), /*           */
            (0, 2), (1, 2), /*   */
        ],
    );
    assert_pixels_with_colour(
        &dst,
        &col,
        &vec![
            (0, 0), /*   */ (2, 0),
            /*   */ (1, 1), (2, 1),
            /*           */ (2, 2),
        ],
    );
}

#[wasm_bindgen_test]
fn canvas_sample() {
    let mut dst = Canvas::new(5, 5);
    let col = ARGBColour::new(255, 255, 0, 0);
    let col_u32 = u32::from(&col);
    let src = vec![
        col_u32, 0,       col_u32, 0,       0,
        0,       col_u32, 0,       0,       col_u32,
        0,       0,       col_u32, 0,       0,
        0,       col_u32, 0,       0,       0,
        0,       0,       0,       0,       0,
    ];

    assert_no_pixels_with_colour(&dst, &col);
    assert!(dst.load_pixels(src));

    assert_eq!(col_u32, dst.sample(0f64, 0f64, UVWrapMode::Clamp));
    assert_eq!(0,       dst.sample(1f64, 1f64, UVWrapMode::Clamp));
    assert_eq!(0,       dst.sample(2f64, 2f64, UVWrapMode::Clamp));

    // (1, 0)
    assert_eq!(0, dst.sample(0.25f64, 0f64, UVWrapMode::Clamp));

    // (1, 1)
    assert_eq!(col_u32, dst.sample(0.25f64, 0.25f64, UVWrapMode::Clamp));

    // (1, 3)
    assert_eq!(col_u32, dst.sample(0.25f64, 0.75f64, UVWrapMode::Clamp));

    // (1, 2)
    assert_eq!(0, dst.sample(0.25f64, 0.5f64, UVWrapMode::Clamp));
}


#[wasm_bindgen_test]
fn drawing_fill_rect() {
    let mut canv = Canvas::new(4, 6);
    let col = ARGBColour::new(255, 255, 0, 0);

    // Assert that no canvas pixels have "col" value
    assert_no_pixels_with_colour(&canv, &col);

    // Draw a rectangle (2, 4, 5, 7): should fill pixels (2,4), (3,4), (2,5), (3,5)
    drawing::rect::fill_rect(&mut canv, &col, 2, 4, 5, 7);

    // Locations which should not be filled
    let ne_idx = vec![
        (0, 0), (1, 0), (2, 0), (3, 0), 
        (0, 1), (1, 1), (2, 1), (3, 1), 
        (0, 2), (1, 2), (2, 2), (3, 2), 
        (0, 3), (1, 3), (2, 3), (3, 3), 
        (0, 4), (1, 4), /*+---------+*/
        (0, 5), (1, 5), /*+---------+*/
    ];

    // Locations which should be filled
    let eq_idx = vec![
        (2, 4), (3, 4), 
        (2, 5), (3, 5), 
    ];

    // Assert all un-filled locations are not set to "col"
    assert_pixels_without_colour(&canv, &col, &ne_idx);

    // Assert all filled locations are set to "col"
    assert_pixels_with_colour(&canv, &col, &eq_idx);

    // Test bounds
    let mut canv = canvas::Canvas::new(4, 4);

    // Assert that no canvas pixels have "col" value
    assert_no_pixels_with_colour(&canv, &col);

    // Draw a rectangle (1, 1, 2, 2): should fill pixels (1,1), (2,1), (1,2), (2,2)
    drawing::rect::fill_rect(&mut canv, &col, 1, 1, 2, 2);

    // Locations which should not be filled
    let ne_idx = vec![
        (0, 0), (1, 0), (2, 0), (3, 0), 
        (0, 1), /*+---------+*/ (3, 1), 
        (0, 2), /*+---------+*/ (3, 2), 
        (0, 3), (1, 3), (2, 3), (3, 3), 
    ];

    // Locations which should be filled
    let eq_idx = vec![
        (1, 1), (2, 1),
        (1, 2), (2, 2),
    ];

    // Assert all un-filled locations are not set to "col"
    assert_pixels_without_colour(&canv, &col, &ne_idx);

    // Assert all filled locations are set to "col"
    assert_pixels_with_colour(&canv, &col, &eq_idx);
}

#[wasm_bindgen_test]
fn drawing_h_line() {
    let mut canv = canvas::Canvas::new(5, 4);
    let col = types::ARGBColour::new(255, 255, 0, 0);

    // Assert that no canvas pixels have "col" value
    assert_no_pixels_with_colour(&canv, &col);

    // Draw a horizontal line on y=1 from x=0 to x=3
    drawing::lines::h_line(&mut canv, &col, 0, 1, 3);

    // Draw a horizontal line on y=3 from x=2 to x=6
    drawing::lines::h_line(&mut canv, &col, 2, 3, 6);

    // Draw a horizontal line on y=4 from x=0 to x=4 (out of canvas bounds)
    drawing::lines::h_line(&mut canv, &col, 0, 4, 4);

    // Locations which should not be filled
    let ne_idx = vec![
        (0, 0), (1, 0), (2, 0), (3, 0), (4, 0),
        /*----------------------------*/(4, 1),
        (0, 2), (1, 2), (2, 2), (3, 2), (4, 2),
        (0, 3), (1, 3),/*--------------------*/
    ];

    // Locations which should be filled
    let eq_idx = vec![
        (0, 1), (1, 1), (2, 1), (3, 1),
        (2, 3), (3, 3), (4, 3),
    ];

    // Assert all un-filled locations are not set to "col"
    assert_pixels_without_colour(&canv, &col, &ne_idx);

    // Assert all filled locations are set to "col"
    assert_pixels_with_colour(&canv, &col, &eq_idx);
}

#[wasm_bindgen_test]
fn drawing_polygon() {
    let mut canv = Canvas::new(4, 6);
    let col = ARGBColour::new(255, 255, 0, 0);

    // Assert that no canvas pixels have "col" value
    assert_no_pixels_with_colour(&canv, &col);

    // Draw a polygon (not closed)
    drawing::shape::polygon(
        &mut canv,
        &col,
        false,
        vec![
            0, 0,
            1, 0,
            1, 2,
            2, 2,
            2, 5,
            0, 5,
        ]
    );

    // Locations which should not be filled
    let ne_idx = vec![
        /* --------+ */ (2, 0), (3, 0),
        (0, 1), /* | */ (2, 1), (3, 1),
        (0, 2), /* +-------| */ (3, 2),
        (0, 3), (1, 3), /* | */ (3, 3),
        (0, 4), (1, 4), /* | */ (3, 4),
        /* ----------------+ */ (3, 5),
    ];

    // Locations which should be filled
    let eq_idx = vec![
        (0, 0), (1, 0),
                (1, 1),
                (1, 2), (2, 2),
                        (2, 3),
                        (2, 4),
        (0, 5), (1, 5), (2, 5),
    ];

    // Assert all un-filled locations are not set to "col"
    assert_pixels_without_colour(&canv, &col, &ne_idx);

    // Assert all filled locations are set to "col"
    assert_pixels_with_colour(&canv, &col, &eq_idx);


    let mut canv = Canvas::new(4, 6);
    let col = ARGBColour::new(255, 255, 0, 0);

    // Assert that no canvas pixels have "col" value
    assert_no_pixels_with_colour(&canv, &col);

    // Draw a polygon (closed)
    drawing::shape::polygon(&mut canv, &col, true, vec![0, 0, 1, 0, 1, 2, 2, 2, 2, 5, 0, 5]);

    let ne_idx = vec![
        /* +-------+ */ (2, 0), (3, 0),
        /* |       | */ (2, 1), (3, 1),
        /* |       +-------| */ (3, 2),
        /* | */ (1, 3), /* | */ (3, 3),
        /* | */ (1, 4), /* | */ (3, 4),
        /* +---------------+ */ (3, 5),
    ];

    let eq_idx = vec![
        (0, 0), (1, 0),
        (0, 1), (1, 1),
        (0, 2), (1, 2), (2, 2),
        (0, 3),         (2, 3),
        (0, 4),         (2, 4),
        (0, 5), (1, 5), (2, 5),
    ];

    // Assert all un-filled locations are not set to "col"
    assert_pixels_without_colour(&canv, &col, &ne_idx);

    // Assert all filled locations are set to "col"
    assert_pixels_with_colour(&canv, &col, &eq_idx);
}

#[wasm_bindgen_test]
fn drawing_filled_polygon() {
    let mut canv = Canvas::new(4, 6);
    let col = ARGBColour::new(255, 255, 0, 0);

    // Assert that no canvas pixels have "col" value
    assert_no_pixels_with_colour(&canv, &col);

    // Draw a polygon (not closed)
    drawing::shape::fill_polygon(
        &mut canv,
        &col,
        vec![
            0, 0,
            1, 0,
            1, 2,
            2, 2,
            2, 5,
            0, 5,
            0, 0,
        ]
    );

    // Locations which should not be filled
    let ne_idx = vec![
        /* +-------+ */ (2, 0), (3, 0),
        /* |       | */ (2, 1), (3, 1),
        /* |       +-------| */ (3, 2),
        /* |               | */ (3, 3),
        /* |               | */ (3, 4),
        /* +---------------+ */ (3, 5),
    ];

    // Locations which should be filled
    let eq_idx = vec![
        (0, 0), (1, 0),
        (0, 1), (1, 1),
        (0, 2), (1, 2), (2, 2),
        (0, 3), (1, 3), (2, 3),
        (0, 4), (1, 4), (2, 4),
        (0, 5), (1, 5), (2, 5),
    ];

    // Assert all un-filled locations are not set to "col"
    assert_pixels_without_colour(&canv, &col, &ne_idx);

    // Assert all filled locations are set to "col"
    assert_pixels_with_colour(&canv, &col, &eq_idx);
}

#[wasm_bindgen_test]
fn drawing_rect() {
    let mut canv = Canvas::new(4, 6);
    let col = ARGBColour::new(255, 255, 0, 0);

    // Assert that no canvas pixels have "col" value
    assert_no_pixels_with_colour(&canv, &col);

    // Draw a rectangle (1, 2, 5, 7)
    drawing::rect::rect(&mut canv, &col, 1, 2, 5, 7);

    // Locations which should not be filled
    let ne_idx = vec![
        (0, 0), (1, 0), (2, 0), (3, 0),
        (0, 1), (1, 1), (2, 1), (3, 1),
        (0, 2), /* +---------------- */
        (0, 3), /* |  */(2, 3), /*   */
        (0, 4), /* |  */(2, 4), /*   */
        (0, 5), /* |  */(2, 5), /*   */
    ];

    // Locations which should be filled
    let eq_idx = vec![
        (1, 2), (2, 2), (3, 2),
        (1, 3), /*           */
        (1, 4), /*           */
        (1, 5), /*           */
    ];

    // Assert all un-filled locations are not set to "col"
    assert_pixels_without_colour(&canv, &col, &ne_idx);

    // Assert all filled locations are set to "col"
    assert_pixels_with_colour(&canv, &col, &eq_idx);

    // Test bounds
    let mut canv = canvas::Canvas::new(4, 4);

    // Assert that no canvas pixels have "col" value
    assert_no_pixels_with_colour(&canv, &col);

    // Draw a rectangle (1, 0, 2, 2)
    drawing::rect::rect(&mut canv, &col, 1, 0, 2, 2);

    // Locations which should not be filled
    let ne_idx = vec![
        (0, 0), /*+---------+*/ (3, 0),
        (0, 1), /*|         |*/ (3, 1),
        (0, 2), /*+---------+*/ (3, 2),
        (0, 3), (1, 3), (2, 3), (3, 3),
    ];

    // Locations which should be filled
    let eq_idx = vec![
        (1, 0), (2, 0),
        (1, 1), (2, 1),
        (1, 2), (2, 2),
    ];

    // Assert all un-filled locations are not set to "col"
    assert_pixels_without_colour(&canv, &col, &ne_idx);

    // Assert all filled locations are set to "col"
    assert_pixels_with_colour(&canv, &col, &eq_idx);
}

#[wasm_bindgen_test]
fn drawing_fill_triangle() {
    let mut canv = Canvas::new(4, 4);
    let col = ARGBColour::new(255, 255, 0, 0);

    // Assert that no canvas pixels have "col" value
    assert_no_pixels_with_colour(&canv, &col);

    // Draw a filled triangle
    drawing::shape::fill_triangle(&mut canv, &col, 0, 0, 3, 3, 0, 3);

    // Locations which should not be filled
    let ne_idx = vec![
        /*+---*/(1, 0), (2, 0), (3, 0),
        /*|#####------*/(2, 1), (3, 1),
        /*|#############------*/(3, 2),
        /*+-------------------------*/
    ];

    // Locations which should be filled
    let eq_idx = vec![
        (0, 0),
        (0, 1), (1, 1),
        (0, 2), (1, 2), (2, 2),
        (0, 3), (1, 3), (2, 3), (3, 3),
    ];

    // Assert all un-filled locations are not set to "col"
    assert_pixels_without_colour(&canv, &col, &ne_idx);

    // Assert all filled locations are set to "col"
    assert_pixels_with_colour(&canv, &col, &eq_idx);

    /*
        (0, 0), (1, 0), (2, 0), (3, 0),
        (0, 1), (1, 1), (2, 1), (3, 1),
        (0, 2), (1, 2), (2, 2), (3, 2),
        (0, 3), (1, 2), (2, 2), (3, 2),
    */
}

#[wasm_bindgen_test]
fn drawing_v_line() {
    let mut canv = canvas::Canvas::new(4, 5);
    let col = types::ARGBColour::new(255, 255, 0, 0);

    // Assert that no canvas pixels have "col" value
    assert_no_pixels_with_colour(&canv, &col);

    // Draw a vertical line on x=1 from y=0 to y=3
    drawing::lines::v_line(&mut canv, &col, 1, 0, 3);

    // Draw a vertical line on x=3 from y=2 to y=6
    drawing::lines::v_line(&mut canv, &col, 3, 2, 6);

    // Draw a vertical line on x=4 from y=0 to y=4 (out of canvas bounds)
    drawing::lines::v_line(&mut canv, &col, 4, 0, 4);

    // Locations which should not be filled
    let ne_idx = vec![
        (0, 0),/*  |  */(2, 0), (3, 0),
        (0, 1),/*  |  */(2, 1), (3, 1),
        (0, 2),/*  |  */(2, 2),/*  |  */
        (0, 3),/*  |  */(2, 3),/*  |  */
        (0, 4), (1, 4), (2, 4),/*  |  */
    ];

    // Locations which should be filled
    let eq_idx = vec![
        (1, 0),
        (1, 1),
        (1, 2), (3, 2),
        (1, 3), (3, 3),
        (3, 4),
    ];

    // Assert all un-filled locations are not set to "col"
    assert_pixels_without_colour(&canv, &col, &ne_idx);

    // Assert all filled locations are set to "col"
    assert_pixels_with_colour(&canv, &col, &eq_idx);
}
