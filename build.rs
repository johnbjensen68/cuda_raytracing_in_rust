use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/kernels/raytrace.cu");

    let gpu = env::var_os("CARGO_FEATURE_GPU").is_some();
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let ptx_path = out_dir.join("raytrace.ptx");

    if !gpu {
        if !ptx_path.exists() {
            fs::write(&ptx_path, "").unwrap();
        }
        println!("cargo:rustc-env=PTX_PATH={}", ptx_path.display());
        return;
    }

    let kernel = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("src/kernels/raytrace.cu");

    let nvcc = which_nvcc();
    match nvcc {
        Some(nvcc) => {
            let status = Command::new(&nvcc)
                .args([
                    "-arch=sm_60",
                    "-O3",
                    "-ptx",
                    "--use_fast_math",
                ])
                .arg(&kernel)
                .arg("-o")
                .arg(&ptx_path)
                .status()
                .expect("failed to invoke nvcc");
            if !status.success() {
                panic!("nvcc failed (exit {:?}) compiling {}", status, kernel.display());
            }
        }
        None => {
            println!(
                "cargo:warning=nvcc not found on PATH; writing empty PTX stub. \
                 The `gpu` feature requires the CUDA toolkit. Runtime kernel \
                 load will fail until you build on a machine with nvcc."
            );
            fs::write(&ptx_path, "").unwrap();
        }
    }

    println!("cargo:rustc-env=PTX_PATH={}", ptx_path.display());
}

fn which_nvcc() -> Option<PathBuf> {
    if let Ok(p) = env::var("NVCC") {
        return Some(PathBuf::from(p));
    }
    for dir in env::var_os("PATH")?.to_string_lossy().split(':') {
        let candidate = PathBuf::from(dir).join("nvcc");
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    for fixed in [
        "/usr/local/cuda/bin/nvcc",
        "/opt/cuda/bin/nvcc",
    ] {
        let p = PathBuf::from(fixed);
        if p.is_file() {
            return Some(p);
        }
    }
    None
}
