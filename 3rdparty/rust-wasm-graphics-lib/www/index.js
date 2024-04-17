import {
  ARGBColour,
  Canvas,
  UVWrapMode,
  UVVertex,
  fill_polygon,
  fill_rect,
  fill_triangle,
  line,
  polygon,
  rect,
  textured_triangle,
} from 'rust-wasm-graphics-lib';

import { memory } from 'rust-wasm-graphics-lib/rust_wasm_graphics_lib_bg';

const SETTINGS = {
  clear_canvas:           true,
  demo_fill_rect:         false,
  demo_fill_polygon:      false,
  demo_fill_triangle:     false,
  demo_fill_triangle_rot: false,
  demo_hlines:            false,
  demo_lines:             false,
  demo_polygon:           false,
  demo_rect:              false,
  demo_rotating_line:     false,
  demo_sprite:            false,
  demo_textured_triangle: false,
  demo_vlines:            false,
};

const SIZE_MULT = 2;
const WIDTH = 256;
const HEIGHT = 256;

const POLY_POINTS = [
  [1.0, 0.0],
  [-0.8090169943749473, 0.5877852522924732],
  [0.30901699437494723, -0.9510565162951536],
  [0.30901699437494745, 0.9510565162951535],
  [-0.8090169943749476, -0.587785252292473],
  [1.0, 0.0],
];
const POLY_POINTS_TRI = [
  [ 0, -1],
  [ 1,  1],
  [-1,  1],
];
const POLY_POINTS_QUAD = [
  [-1, -1],
  [ 1, -1],
  [-1,  1],
  [ 1,  1],
];
const POLY_SIZE = WIDTH / 3;

const clear_colour = ARGBColour.new(255, 0, 0, 0);
const draw_colour = ARGBColour.new(255, 0, 0, 0);

const rust_canvas = Canvas.new(WIDTH, HEIGHT);
const height      = rust_canvas.height();
const width       = rust_canvas.width();

const canvas = document.getElementById('main-canvas');
canvas.height = height * SIZE_MULT;
canvas.width  = width  * SIZE_MULT;

const canvas_back = document.createElement('canvas');
canvas_back.height = height;
canvas_back.width  = width;

const ctx = canvas.getContext('2d');
const ctx_back = canvas_back.getContext('2d');
ctx.imageSmoothingEnabled = false;

let counter = 0;

let sprite_canv = null;

function getImageData(url, cb) {
  const img = new Image();
  img.onload = () => {
    const canv = document.createElement('canvas');
    const ctx = canv.getContext('2d');
    canv.width = img.width;
    canv.height = img.height;
    ctx.drawImage(img, 0, 0);
    const id = ctx.getImageData(0, 0, img.width, img.height);
    const buf32 = new Uint32Array(id.data.buffer);
    cb(buf32, img.width, img.height);
  };
  img.src = url;
}

getImageData(
  'i/ferris.png',
  (buf, w, h) => {
    console.log('Loaded image:', w, h);
    sprite_canv = Canvas.new(w, h);
    if (sprite_canv.load_pixels(buf)) {
      console.log('Image loaded into Canvas');
    }
  },
);


/*
 * Demonstration renderers
 */
const demo_fill_polygon = () => {
  let nr = Math.floor(((Math.sin(counter * 0.7) + 1) / 2) * 255);
  let ng = Math.floor(((Math.cos(counter * 0.5) + 1) / 2) * 255);
  let nb = Math.floor(((Math.sin(counter * 0.2) + 1) / 2) * 255);
  draw_colour.r = nr;
  draw_colour.g = ng;
  draw_colour.b = nb;

  const points = POLY_POINTS
    .map((pt) => [
      ((pt[0] * POLY_SIZE * Math.cos(counter)) - (pt[1] * POLY_SIZE * Math.sin(counter))) + (WIDTH  / 2),
      ((pt[0] * POLY_SIZE * Math.sin(counter)) + (pt[1] * POLY_SIZE * Math.cos(counter))) + (HEIGHT / 2),
    ])
    .reduce((a, pt) => a.concat(pt), []);

  fill_polygon(
    rust_canvas,
    draw_colour,
    points,
  );
};
const demo_fill_rect = () => {
  let nr, ng, nb;

  nr = Math.floor(((Math.sin(counter * 1.5) + 1) / 2) * 255);
  ng = Math.floor(((Math.cos(counter * 1.7) + 1) / 2) * 255);
  nb = Math.floor(((Math.sin(counter * 1.3) + 1) / 2) * 255);
  draw_colour.r = nr;
  draw_colour.g = ng;
  draw_colour.b = nb;

  let rectx = Math.floor(((Math.sin(counter * 1.3) + 1) / 2) * width  / 2);
  let recty = Math.floor(((Math.cos(counter * 1.5) + 1) / 2) * height / 2);
  let rectw = Math.floor(((Math.sin(counter * 2.1) + 1) / 2) * width  / 2) + 16;
  let recth = Math.floor(((Math.cos(counter * 1.8) + 1) / 2) * height / 2) + 16;

  fill_rect(
    rust_canvas,
    draw_colour,
    rectx,
    recty,
    rectx + rectw,
    recty + recth,
  );
};
const demo_fill_triangle = () => {
  let nr = Math.floor(((Math.sin(counter * 0.25) + 1) / 2) * 255);
  let ng = Math.floor(((Math.cos(counter * 0.11) + 1) / 2) * 255);
  let nb = Math.floor(((Math.sin(counter * 0.81) + 1) / 2) * 255);
  draw_colour.r = nr;
  draw_colour.g = ng;
  draw_colour.b = nb;

  fill_triangle(
    rust_canvas,
    draw_colour,
    Math.floor(((Math.cos(counter * 1.2) + 1) / 2) * WIDTH),
    Math.floor(((Math.sin(counter * 1.5) + 1) / 2) * HEIGHT),
    Math.floor(((Math.cos(counter * 1.5) + 1) / 2) * WIDTH),
    Math.floor(((Math.sin(counter * 1.2) + 1) / 2) * HEIGHT),
    Math.floor(((Math.cos(counter * 1.9) + 1) / 2) * WIDTH),
    Math.floor(((Math.sin(counter * 1.7) + 1) / 2) * HEIGHT),
  );
};
const demo_fill_triangle_rot = () => {
  let nr = Math.floor(((Math.sin(counter * 0.375) + 1) / 2) * 255);
  let ng = Math.floor(((Math.cos(counter * 0.73) + 1) / 2) * 255);
  let nb = Math.floor(((Math.sin(counter * 0.28) + 1) / 2) * 255);
  draw_colour.r = nr;
  draw_colour.g = ng;
  draw_colour.b = nb;

  const pts = POLY_POINTS_TRI
    .map((pt) => [
      ((pt[0] * POLY_SIZE / 2 * Math.cos(counter * 1.8)) - (pt[1] * POLY_SIZE / 2 * Math.sin(counter * 1.8))) + (WIDTH  / 2),
      ((pt[0] * POLY_SIZE / 2 * Math.sin(counter * 1.8)) + (pt[1] * POLY_SIZE / 2 * Math.cos(counter * 1.8))) + (HEIGHT / 2),
    ])
    .reduce((a, pt) => a.concat(pt), []);

  fill_triangle(
    rust_canvas,
    draw_colour,
    ...pts
  );
};
const demo_hlines = () => {
  let nr = Math.floor(((Math.sin(counter * 1.1) + 1) / 2) * 255);
  let ng = Math.floor(((Math.cos(counter * 0.8) + 1) / 2) * 255);
  let nb = Math.floor(((Math.sin(counter * 1.4) + 1) / 2) * 255);
  draw_colour.r = nr;
  draw_colour.g = ng;
  draw_colour.b = nb;

  let x1 = Math.floor(((Math.sin(counter * 0.3) + 1) / 2) * width);
  let x2 = Math.floor(((Math.sin(counter * 1.1) + 1) / 2) * width) + 16;
  let y = Math.floor(((Math.cos(counter * 0.5) + 1) / 2) * height);

  line(rust_canvas, draw_colour, x1, y, x2, y);
};
const demo_lines = () => {
  let nr = Math.floor(((Math.sin(counter * 1.8) + 1) / 2) * 255);
  let ng = Math.floor(((Math.cos(counter * 0.2) + 1) / 2) * 255);
  let nb = Math.floor(((Math.sin(counter * 0.7) + 1) / 2) * 255);
  draw_colour.r = nr;
  draw_colour.g = ng;
  draw_colour.b = nb;

  let x1 = Math.floor(((Math.sin(counter * 0.8) + 1) / 2) * width);
  let x2 = Math.floor(((Math.cos(counter * 1.1) + 1) / 2) * width) + 16;
  let y1 = Math.floor(((Math.sin(counter * 0.5) + 1) / 2) * height);
  let y2 = Math.floor(((Math.cos(counter * 1.2) + 1) / 2) * height) + 16;

  line(rust_canvas, draw_colour, x1, y1, x2, y2);
};
const demo_polygon = () => {
  let nr = Math.floor(((Math.sin(counter * 1.4) + 1) / 2) * 255);
  let ng = Math.floor(((Math.cos(counter * 0.6) + 1) / 2) * 255);
  let nb = Math.floor(((Math.sin(counter * 0.9) + 1) / 2) * 255);
  draw_colour.r = nr;
  draw_colour.g = ng;
  draw_colour.b = nb;

  const points = POLY_POINTS
    .map((pt) => [
      ((pt[0] * POLY_SIZE * Math.cos(counter)) - (pt[1] * POLY_SIZE * Math.sin(counter))) + (WIDTH  / 2),
      ((pt[0] * POLY_SIZE * Math.sin(counter)) + (pt[1] * POLY_SIZE * Math.cos(counter))) + (HEIGHT / 2),
    ])
    .reduce((a, pt) => a.concat(pt), []);

  polygon(
    rust_canvas,
    draw_colour,
    false,
    points,
  );
};
const demo_rect = () => {
  let nr, ng, nb;

  nr = Math.floor(((Math.cos(counter * 1.1) + 1) / 2) * 255);
  ng = Math.floor(((Math.sin(counter * 1.4) + 1) / 2) * 255);
  nb = Math.floor(((Math.cos(counter * 2.3) + 1) / 2) * 255);
  draw_colour.r = nr;
  draw_colour.g = ng;
  draw_colour.b = nb;

  let rectx = Math.floor(((Math.cos(counter * 1.8) + 1) / 2) * width  / 2);
  let recty = Math.floor(((Math.sin(counter * 2.2) + 1) / 2) * height / 2);
  let rectw = Math.floor(((Math.cos(counter * 1.1) + 1) / 2) * width  / 2) + 16;
  let recth = Math.floor(((Math.sin(counter * 1.4) + 1) / 2) * height / 2) + 16;

  rect(
    rust_canvas,
    draw_colour,
    rectx,
    recty,
    rectx + rectw,
    recty + recth,
  );
};
const demo_rotating_line = () => {
  let nr = Math.floor(((Math.sin(counter * 1.4) + 1) / 2) * 255);
  let ng = Math.floor(((Math.cos(counter * 0.6) + 1) / 2) * 255);
  let nb = Math.floor(((Math.sin(counter * 0.9) + 1) / 2) * 255);
  draw_colour.r = nr;
  draw_colour.g = ng;
  draw_colour.b = nb;

  let x1 = WIDTH / 2;
  let y1 = HEIGHT / 2;
  let x2 = (WIDTH / 2) + (100 * Math.cos(-counter));
  let y2 = (HEIGHT / 2) + (100 * Math.sin(-counter));

  line(rust_canvas, draw_colour, x1, y1, x2, y2);
};
const demo_vlines = () => {
  let nr = Math.floor(((Math.sin(counter * 1.2) + 1) / 2) * 255);
  let ng = Math.floor(((Math.cos(counter * 1.3) + 1) / 2) * 255);
  let nb = Math.floor(((Math.sin(counter * 1.8) + 1) / 2) * 255);
  draw_colour.r = nr;
  draw_colour.g = ng;
  draw_colour.b = nb;

  let y1 = Math.floor(((Math.sin(counter * 0.5) + 1) / 2) * height);
  let y2 = Math.floor(((Math.sin(counter * 1.2) + 1) / 2) * height) + 16;
  let x = Math.floor(((Math.cos(counter * 0.3) + 1) / 2) * width);

  line(rust_canvas, draw_colour, x, y1, x, y2);
};
const demo_draw_sprite = () => {
  if (!sprite_canv) {
    return;
  }
  const x = Math.floor(((Math.sin(counter * 1.57) + 1) / 2) * (width  - sprite_canv.width()));
  const y = Math.floor(((Math.cos(counter * 1.32) + 1) / 2) * (height - sprite_canv.height()));
  rust_canvas.draw_canvas(sprite_canv, x, y);
};
const demo_textured_triangle = () => {
  if (!sprite_canv) {
    return;
  }
  const points = POLY_POINTS_QUAD
    .map((pt) => [
      ((pt[0] * POLY_SIZE * Math.cos(counter)) - (pt[1] * POLY_SIZE * Math.sin(counter))) + (WIDTH  / 2),
      ((pt[0] * POLY_SIZE * Math.sin(counter)) + (pt[1] * POLY_SIZE * Math.cos(counter))) + (HEIGHT / 2),
    ]);

  const uvscale = (((Math.sin(counter) + 1) / 2) * 10) + 0.75;
  const a = UVVertex.new(points[0][0], points[0][1], -uvscale, -uvscale);
  const b = UVVertex.new(points[1][0], points[1][1],  uvscale, -uvscale);
  const c = UVVertex.new(points[2][0], points[2][1], -uvscale,  uvscale);
  const d = UVVertex.new(points[3][0], points[3][1],  uvscale,  uvscale);
  textured_triangle(rust_canvas, sprite_canv, a, b, c, UVWrapMode.Wrap);
  textured_triangle(rust_canvas, sprite_canv, b, c, d, UVWrapMode.Wrap);
};


const renderLoop = () => {
  //debugger;

  if (SETTINGS.clear_canvas) { rust_canvas.clear(clear_colour) };

  if (SETTINGS.demo_fill_rect)         { demo_fill_rect();         }
  if (SETTINGS.demo_rect)              { demo_rect();              }
  if (SETTINGS.demo_hlines)            { demo_hlines();            }
  if (SETTINGS.demo_vlines)            { demo_vlines();            }
  if (SETTINGS.demo_rotating_line)     { demo_rotating_line();     }
  if (SETTINGS.demo_lines)             { demo_lines();             }
  if (SETTINGS.demo_fill_polygon)      { demo_fill_polygon();      }
  if (SETTINGS.demo_polygon)           { demo_polygon();           }
  if (SETTINGS.demo_fill_triangle)     { demo_fill_triangle();     }
  if (SETTINGS.demo_fill_triangle_rot) { demo_fill_triangle_rot(); }
  if (SETTINGS.demo_sprite)            { demo_draw_sprite();       }
  if (SETTINGS.demo_textured_triangle) { demo_textured_triangle(); }

  drawBuffer();
  counter += 0.01;

  requestAnimationFrame(renderLoop);
};

const drawBuffer = () => {
  const buffer = rust_canvas.buffer_ptr();
  const buff_u8 = new Uint8ClampedArray(memory.buffer, buffer, width * height * 4);
  const img = new ImageData(buff_u8, width, height);
  ctx_back.putImageData(img, 0, 0);
  ctx.drawImage(canvas_back, 0, 0, width, height, 0, 0, width * SIZE_MULT, height * SIZE_MULT);
};

renderLoop();


/*
 * Event handlers for UI settings
 */
document.getElementById('cb_clear').addEventListener('change', (evt) => {
  SETTINGS.clear_canvas = evt.target.checked;
});
document.getElementById('cb_demo_fill_polygon').addEventListener('change', (evt) => {
  SETTINGS.demo_fill_polygon = evt.target.checked;
});
document.getElementById('cb_demo_fill_rect').addEventListener('change', (evt) => {
  SETTINGS.demo_fill_rect = evt.target.checked;
});
document.getElementById('cb_demo_fill_triangle').addEventListener('change', (evt) => {
  SETTINGS.demo_fill_triangle = evt.target.checked;
});
document.getElementById('cb_demo_fill_triangle_rot').addEventListener('change', (evt) => {
  SETTINGS.demo_fill_triangle_rot = evt.target.checked;
});
document.getElementById('cb_demo_hlines').addEventListener('change', (evt) => {
  SETTINGS.demo_hlines = evt.target.checked;
});
document.getElementById('cb_demo_lines').addEventListener('change', (evt) => {
  SETTINGS.demo_lines = evt.target.checked;
});
document.getElementById('cb_demo_polygon').addEventListener('change', (evt) => {
  SETTINGS.demo_polygon = evt.target.checked;
});
document.getElementById('cb_demo_rect').addEventListener('change', (evt) => {
  SETTINGS.demo_rect = evt.target.checked;
});
document.getElementById('cb_demo_rotating_line').addEventListener('change', (evt) => {
  SETTINGS.demo_rotating_line = evt.target.checked;
});
document.getElementById('cb_demo_sprite').addEventListener('change', (evt) => {
  SETTINGS.demo_sprite = evt.target.checked;
});
document.getElementById('cb_demo_textured_triangle').addEventListener('change', (evt) => {
  SETTINGS.demo_textured_triangle = evt.target.checked;
});
document.getElementById('cb_demo_vlines').addEventListener('change', (evt) => {
  SETTINGS.demo_vlines = evt.target.checked;
});
