# mandelbrot

Parallel Mandelbrot set fractal renderer.

Features:
- Multi-threaded rendering using `crossbeam` for scoped threads
- Automatic CPU core detection with `num_cpus`
- Complex number math using `num::Complex`
- PNG image output using the `image` crate

## Usage

```bash
cargo run -p mandelbrot -- <filename> <pixels> <upper_left> <lower_right>
```

Example:
```bash
cargo run -p mandelbrot -- mandel.png 1000x750 -1.20,0.35 -1,0.20
```

## Architecture

- `library/common/environment.rs` - command-line parsing utilities
- `library/math/mandelbrot.rs` - core rendering logic, escape-time algorithm, PNG encoding
