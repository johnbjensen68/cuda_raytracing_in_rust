# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this is

A Rust translation of the C++ ray tracer from *Ray Tracing: The Next Week* (book 2 of the Shirley/Black/Hollasch series). This repo's listings are the source of truth for the companion site at https://ray-tracing-the-next-week-in-rust.vercel.app/ — when changing code, prefer keeping each file's structure close to the book's corresponding chapter so the side-by-side C++/Rust presentation still reads cleanly.

The Cargo package is named `cuda_raytracing_in_rust`, but there is currently no CUDA/GPU code — it's a CPU-only `std::rc::Rc`-based implementation. The name reflects an intended direction, not the current state.

## Build and run

```bash
cargo run --release > image.ppm   # render currently selected scene to PPM
cargo build --release             # build only
cargo check                       # fast type-check
```

The renderer writes the **PPM image to stdout** and **progress (scanlines remaining) to stderr** — never `println!` debug output, it will corrupt the image. Redirect stdout to a file (or pipe to an image viewer) when running.

There are no tests in this repo (`cargo test` runs zero tests).

## Selecting a scene

`fn main()` in `src/main.rs` is a `match` literal that picks one of the scene functions (1=bouncing_spheres, 2=checkered_spheres, 3=earth, 4=perlin_spheres, 5=quads, 6=simple_light, 7=cornell_box, 8=cornell_smoke). To render a different scene, **edit the integer in `match N {`** and rerun — there is no CLI argument. The `earth` scene reads `earthmap.jpg` from the working directory.

## Architecture

The core abstraction is the `Hittable` trait (`src/hittable.rs`):

```rust
trait Hittable {
    fn hit(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord>;
    fn bounding_box(&self) -> Aabb;
}
```

Everything renderable implements it: primitives (`Sphere`, `Quad`), aggregates (`HittableList`, `BvhNode`), transforms (`Translate`, `RotateY`), and volumes (`ConstantMedium`). Transforms are **wrapper `Hittable`s** that take a `Box<dyn Hittable>` and modify the ray on the way in / the hit record on the way out — the same pattern as the book.

`Camera::render(&dyn Hittable)` owns the outer render loop: for each pixel, jitter `samples_per_pixel` rays, accumulate via `ray_color`, gamma-correct and write through `color::write_color`. `ray_color` recurses up to `max_depth`, calling `Material::scatter` + `Material::emitted` on each hit.

Materials (`src/material.rs`) — `Lambertian`, `Metal`, `Dielectric`, `DiffuseLight`, `Isotropic` — implement the `Material` trait. Diffuse materials can be backed by a `Texture` (`SolidColor`, `CheckerTexture`, `NoiseTexture` w/ Perlin, `ImageTexture`).

Acceleration: `BvhNode::new(HittableList)` consumes the list and returns a BVH tree. Scenes that need speed (e.g. `bouncing_spheres`) wrap their world in `BvhNode`; smaller scenes use `HittableList` directly.

### Sharing model

All shared ownership is `Rc<T>` (single-threaded). There is **no `Arc` and no Rayon** — adding parallelism means changing every `Rc` in materials/textures/hittables to `Arc` plus a `Send + Sync` audit. Treat this as a deliberate design boundary, not an oversight.

### Coordinate / time conventions

- Right-handed: camera looks down `-w`, `+y` is up.
- Rays carry a `time` field; `Sphere::new_moving` and `Camera::get_ray` (jittered ray time per sample) implement motion blur.
- `Interval` (`src/interval.rs`) and `Aabb` (`src/aabb.rs`) are the small value types that thread through `hit` calls and BVH construction.

## Style notes when editing

- The book's C++ uses shared_ptr; the Rust port uses `Rc<dyn Trait>`. When adding a new material/texture/hittable, follow that pattern (`Rc::new(MyThing::new(...))`) so call sites match the rest of the codebase.
- Scene-setup functions construct a `HittableList` (often wrapped in `BvhNode`), build a `Camera` with the appropriate `lookfrom`/`lookat`/`vfov`/`background`, and call `cam.render(&world)`. New scenes should follow this shape and be wired into the `match` in `main`.
- Background color is passed into the camera per-scene; light-emitting scenes (Cornell box, simple_light, cornell_smoke) use `Color::new(0.0, 0.0, 0.0)` so emission dominates.
