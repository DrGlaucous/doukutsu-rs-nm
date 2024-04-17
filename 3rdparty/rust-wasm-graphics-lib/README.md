# Simple Rust and WebAssembly Graphics Library

A simple graphics library written in Rust for WebAssembly targets. Used to demonstrate benchmarks, unit tests and simple graphics routines.

This library is not hardware accelerated and therefore should not be used for anything serious. It was built for fun and learning!

This project is built with: -

  - [`wasm-pack`](https://github.com/rustwasm/wasm-pack)
  - [`wasm-bindgen`](https://github.com/rustwasm/wasm-bindgen) for communicating
    between WebAssembly and JavaScript.
  - [`console_error_panic_hook`](https://github.com/rustwasm/console_error_panic_hook)
    for logging panic messages to the developer console.
  - [`wee_alloc`](https://github.com/rustwasm/wee_alloc), an allocator optimized
    for small code size.

# Usage

Build with `wasm-pack`: -

```shell
wasm-pack build
```

# Demo

[Online demo](https://www.polaris64.net/resources/programming/rust_wasm_graphics_lib/)
