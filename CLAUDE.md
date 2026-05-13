# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this is

A Rust translation of the C++ ray tracer from *Ray Tracing: The Next Week* (book 2 of the Shirley/Black/Hollasch series). This repo's listings are the source of truth for the companion site at https://ray-tracing-the-next-week-in-rust.vercel.app/ — when changing code, prefer keeping each file's structure close to the book's corresponding chapter so the side-by-side C++/Rust presentation still reads cleanly.

There are two render paths:
- **CPU (default)** — single-threaded `Rc<dyn Trait>` implementation, f64, source of truth for the book listings.
- **GPU (`--features gpu`)** — CUDA kernel via `cudarc`, f32, scoped to sphere primitives + Lambertian/Metal/Dielectric + SolidColor/depth-1-Checker textures + BVH + motion blur + depth of field. Not all CPU scenes have a GPU counterpart yet.

## Build and run

```bash
cargo run --release > image.ppm                  # CPU render (default)
cargo run --release --features gpu > image.ppm   # GPU render (scenes 9 and 10 only)
cargo build --release                            # build only
cargo check                                      # fast type-check
```

The renderer writes the **PPM image to stdout** and **progress to stderr** — never `println!` debug output, it will corrupt the image. Redirect stdout to a file (or pipe to an image viewer) when running.

There are no tests in this repo (`cargo test` runs zero tests).

### GPU build prerequisites

- **CUDA toolkit with `nvcc`** on PATH (or set `NVCC=/path/to/nvcc`). `build.rs` shells out to `nvcc -arch=sm_60 -O3 -ptx --use_fast_math` to compile `src/kernels/raytrace.cu` to `$OUT_DIR/raytrace.ptx`. If `nvcc` is missing, the build prints a warning and writes an empty PTX stub so the Rust side still compiles — but kernel load will fail at runtime.
- **Nvidia GPU + driver** at run time. `cudarc` is configured with `fallback-dynamic-loading`, so the binary links without CUDA libraries present, but `CudaContext::new` will fail if the driver isn't installed.
- The `cudarc` dependency is feature-gated; default builds pull no CUDA crates.
- CUDA toolkit version is pinned via the `cuda-12080` cudarc feature (CUDA 12.8). If your toolkit is older, change it in `Cargo.toml` to the matching `cuda-NNNNN` feature.

## Selecting a scene

`fn main()` in `src/main.rs` is a `match` literal that picks one of the scene functions:
- 1=bouncing_spheres, 2=checkered_spheres, 3=earth, 4=perlin_spheres, 5=quads, 6=simple_light, 7=cornell_box, 8=cornell_smoke (CPU)
- 9=bouncing_spheres_gpu, 10=checkered_spheres_gpu (only with `--features gpu`)

To render a different scene, **edit the integer in `match N {`** and rerun — there is no CLI argument. The `earth` scene reads `earthmap.jpg` from the working directory.

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

CPU path: all shared ownership is `Rc<T>` (single-threaded). There is **no `Arc` and no Rayon** on the CPU — adding CPU parallelism means changing every `Rc` in materials/textures/hittables to `Arc` plus a `Send + Sync` audit. Treat this as a deliberate design boundary, not an oversight.

GPU path: completely separate. The GPU scene is built via `gpu::GpuSceneBuilder` (in `src/gpu/scene.rs`) into flat `#[repr(C)]` arrays (`GpuSphere`, `GpuMaterial`, `GpuTexture`, linearized `GpuBvhNode` tree) — no trait objects, no `Rc`. The kernel in `src/kernels/raytrace.cu` mirrors `Sphere::hit`/`Aabb::hit`/material scatter/`ray_color` in f32 CUDA C++. BVH traversal is iterative with a 64-deep node stack. Kernel struct layouts must byte-match the Rust structs in `src/gpu/scene.rs` — there are `static_assert(sizeof(...))` checks on the device side.

### Coordinate / time conventions

- Right-handed: camera looks down `-w`, `+y` is up.
- Rays carry a `time` field; `Sphere::new_moving` and `Camera::get_ray` (jittered ray time per sample) implement motion blur.
- `Interval` (`src/interval.rs`) and `Aabb` (`src/aabb.rs`) are the small value types that thread through `hit` calls and BVH construction.

## Style notes when editing

- The book's C++ uses shared_ptr; the Rust port uses `Rc<dyn Trait>`. When adding a new material/texture/hittable, follow that pattern (`Rc::new(MyThing::new(...))`) so call sites match the rest of the codebase.
- Scene-setup functions construct a `HittableList` (often wrapped in `BvhNode`), build a `Camera` with the appropriate `lookfrom`/`lookat`/`vfov`/`background`, and call `cam.render(&world)`. New scenes should follow this shape and be wired into the `match` in `main`.
- Background color is passed into the camera per-scene; light-emitting scenes (Cornell box, simple_light, cornell_smoke) use `Color::new(0.0, 0.0, 0.0)` so emission dominates.

## Extending the GPU path

Currently scoped to spheres + Lambertian/Metal/Dielectric + SolidColor/depth-1 Checker. To add a feature:
- **New material kind:** add a `MAT_*` constant in `src/gpu/scene.rs`, add a builder method, and add a branch in `material_scatter` in `src/kernels/raytrace.cu`.
- **New texture kind:** same as above with `TEX_*`. CheckerTexture children are constrained to SolidColor — relaxing that means iterative or fixed-depth lookup in `texture_value`.
- **New primitive (e.g. Quad):** add a `GpuQuad` array + index, extend `GpuBvhNode::leaf_idx` to discriminate primitive type (sign bit or separate tag field), add a `quad_hit` and dispatch in the BVH leaf branch.
- **Emission (`DiffuseLight`):** the iterative `ray_color` already accumulates `attenuation`; add an `emitted` lookup that adds `attenuation * emission` before the scatter step (mirror `src/camera.rs:ray_color`).
- **Layout sync:** any change to a `#[repr(C)]` struct in `src/gpu/scene.rs` must be mirrored in the matching CUDA struct in `src/kernels/raytrace.cu` and the `static_assert(sizeof(...))` updated.
