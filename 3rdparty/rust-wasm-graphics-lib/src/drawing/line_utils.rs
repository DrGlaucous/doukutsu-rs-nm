use crate::canvas::Canvas;

pub fn plot_line_low(c: &mut Canvas, col: u32, x1: usize, y1: usize, x2: usize, y2: usize) {
    let dx: isize = (x2 - x1) as isize;
    let mut dy: isize = (y2 - y1) as isize;
    let mut yi: isize = 1isize;
    if dy < 0 {
        yi = -1isize;
        dy = -dy;
    }
    let mut d: isize = 2isize * (dy as isize) - (dx as isize);
    let mut y: isize = y1 as isize;
    for x in x1..=x2 {
        if y < 0 { break; }
        if y >= c.height() as isize { break; }
        let idx = c.buffer_index(x, y as usize);
        c.buffer_mut()[idx] = col;
        if d > 0 {
            y += yi;
            d -= 2 * dx;
        }
        d += 2 * dy;
    }
}

pub fn plot_line_high(c: &mut Canvas, col: u32, x1: usize, y1: usize, x2: usize, y2: usize) {
    let mut dx: isize = (x2 - x1) as isize;
    let dy: isize = (y2 - y1) as isize;
    let mut xi: isize = 1isize;
    if dx < 0 {
        xi = -1isize;
        dx = -dx;
    }
    let mut d: isize = 2isize * (dx as isize) - (dy as isize);
    let mut x: isize = x1 as isize;
    for y in y1..=y2 {
        if x < 0 { break; }
        if x >= c.width() as isize { break; }
        let idx = c.buffer_index(x as usize, y);
        c.buffer_mut()[idx] = col;
        if d > 0 {
            x += xi;
            d -= 2 * dy;
        }
        d += 2 * dx;
    }
}

