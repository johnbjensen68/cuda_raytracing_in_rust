use std::io::{self, Write};

use cudarc::driver::{CudaContext, DriverError, LaunchConfig, PushKernelArg};
use cudarc::nvrtc::Ptx;

use crate::gpu::scene::{GpuCamera, GpuScene};

const PTX_SRC: &str = include_str!(env!("PTX_PATH"));

pub fn render_gpu(scene: GpuScene, cam: GpuCamera) -> Result<(), DriverError> {
    let ctx = CudaContext::new(0)?;
    let stream = ctx.default_stream();

    let module = ctx.load_module(Ptx::from_src(PTX_SRC))?;
    let func = module.load_function("render")?;

    let d_spheres = stream.clone_htod(&scene.spheres[..])?;
    let d_materials = stream.clone_htod(&scene.materials[..])?;
    let d_textures = stream.clone_htod(&scene.textures[..])?;
    let d_bvh = stream.clone_htod(&scene.bvh[..])?;

    let pixel_count = (cam.image_width as usize) * (cam.image_height as usize);
    let mut d_out = stream.alloc_zeros::<u8>(pixel_count * 3)?;

    let n_spheres = scene.spheres.len() as u32;
    let bvh_root = scene.bvh_root;
    let seed: u64 = 0xC0FFEEC0FFEEC0FFu64;

    let cfg = LaunchConfig {
        grid_dim: (
            (cam.image_width + 15) / 16,
            (cam.image_height + 15) / 16,
            1,
        ),
        block_dim: (16, 16, 1),
        shared_mem_bytes: 0,
    };

    let mut builder = stream.launch_builder(&func);
    builder
        .arg(&mut d_out)
        .arg(&d_spheres)
        .arg(&n_spheres)
        .arg(&d_materials)
        .arg(&d_textures)
        .arg(&d_bvh)
        .arg(&bvh_root)
        .arg(&cam)
        .arg(&seed);
    unsafe { builder.launch(cfg)?; }

    let host_out: Vec<u8> = stream.clone_dtoh(&d_out)?;
    stream.synchronize()?;

    write_ppm(cam.image_width, cam.image_height, &host_out);
    Ok(())
}

fn write_ppm(width: u32, height: u32, rgb: &[u8]) {
    let stdout = io::stdout();
    let mut out = stdout.lock();
    writeln!(out, "P3\n{} {}\n255", width, height).expect("write ppm header");
    for j in (0..height).rev() {
        for i in 0..width {
            let idx = ((j * width + i) * 3) as usize;
            let r = rgb[idx];
            let g = rgb[idx + 1];
            let b = rgb[idx + 2];
            writeln!(out, "{} {} {}", r, g, b).expect("write ppm pixel");
        }
    }
    eprintln!("\nDone.");
}
