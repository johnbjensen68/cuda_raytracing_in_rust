# code_for_next_week_in_rust

A Rust implementation of the ray tracer from [*Ray Tracing: The Next Week*](https://raytracing.github.io/books/RayTracingTheNextWeek.html) by Peter Shirley, Trevor David Black, and Steve Hollasch — the second book in the [Ray Tracing in One Weekend](https://raytracing.github.io/) series.

This repo is a learning exercise for the author: working through the C++ source in the book and translating each idea into idiomatic Rust.

## Companion site

The code in this repo is the source of truth for the listings shown at [ray-tracing-the-next-week-in-rust.vercel.app](https://ray-tracing-the-next-week-in-rust.vercel.app/), which presents the book's narrative with the Rust translations alongside the original C++. The site's MDX content lives in a sibling repository, [ray-tracing-the-next-week-in-rust](https://github.com/johnbjensen68/ray-tracing-the-next-week-in-rust).

## Running it

```bash
cargo run --release > image.ppm
```

`fn main()` in `src/main.rs` is a small `match` that picks which scene to render. Edit the match value (1 = bouncing spheres, …, 9 = the final composite scene) and re-run.

## License

The original book is released under [CC0](https://github.com/RayTracing/raytracing.github.io/blob/release/COPYING.txt); this Rust translation follows the same license.
