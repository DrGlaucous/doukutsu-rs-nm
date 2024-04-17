use wasm_bindgen::prelude::*;

#[wasm_bindgen]
/// A 32-bit ARGB colour
pub struct ARGBColour {
    pub a: u8,
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[wasm_bindgen]
impl ARGBColour {

    /// Creates a new ARGBColour from ARGB u8 components
    pub fn new(a: u8, r: u8, g: u8, b: u8) -> Self {
        Self { a, r, g, b }
    }
}

impl From<&ARGBColour> for u32 {
    fn from(x: &ARGBColour) -> u32 {
        u32::from(x.a) << 24 | u32::from(x.r) << 16 | u32::from(x.g) << 8 | u32::from(x.b)
    }
}


#[wasm_bindgen]
#[derive(Clone, Copy)]
/// A single vertex containing both an (X,Y) co-ordinate and a (U,V) co-ordinate
pub struct UVVertex {
    pub x: isize,
    pub y: isize,
    pub u: f64,
    pub v: f64,
}

#[wasm_bindgen]
impl UVVertex {

    /// Creates a new UVVertex from (X,Y) and (U,V) co-ordinates
    pub fn new(x: isize, y: isize, u: f64, v: f64) -> Self {
        Self { x, y, u, v }
    }
}

#[wasm_bindgen]
#[derive(Clone, Copy)]
/// Type of wrapping to use for U,V co-ordinates outside of the [0,1] range.
///
///   - `Clamp`: co-ordinates are clamped to 0 or 1
///   - `Wrap`: co-ordinates are wrapped (e.g. 1.75 -> 0.75)
pub enum UVWrapMode {
    Clamp,
    Wrap,
}
