use crate::color::Color;
use crate::vec3::{self, Point3, Vec3};

pub const TEX_SOLID: u32 = 0;
pub const TEX_CHECKER: u32 = 1;

pub const MAT_LAMBERTIAN: u32 = 0;
pub const MAT_METAL: u32 = 1;
pub const MAT_DIELECTRIC: u32 = 2;

#[repr(C)]
#[derive(Copy, Clone, Default, Debug)]
pub struct GpuVec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[repr(C)]
#[derive(Copy, Clone, Default, Debug)]
pub struct GpuAabb {
    pub min: GpuVec3,
    pub max: GpuVec3,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct GpuSphere {
    pub center0: GpuVec3,
    pub center1: GpuVec3,
    pub radius: f32,
    pub mat_idx: u32,
    pub is_moving: u32,
    pub _pad: u32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct GpuMaterial {
    pub kind: u32,
    pub tex_idx: u32,
    pub fuzz: f32,
    pub ior: f32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct GpuTexture {
    pub kind: u32,
    pub color: GpuVec3,
    pub inv_scale: f32,
    pub tex_a: u32,
    pub tex_b: u32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct GpuBvhNode {
    pub bbox: GpuAabb,
    pub left: u32,
    pub right: u32,
    pub leaf_idx: i32,
    pub _pad: u32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct GpuCamera {
    pub origin: GpuVec3,
    pub lower_left_corner: GpuVec3,
    pub horizontal: GpuVec3,
    pub vertical: GpuVec3,
    pub u: GpuVec3,
    pub v: GpuVec3,
    pub background: GpuVec3,
    pub lens_radius: f32,
    pub image_width: u32,
    pub image_height: u32,
    pub samples_per_pixel: u32,
    pub max_depth: u32,
    pub _pad: u32,
}

unsafe impl cudarc::driver::DeviceRepr for GpuCamera {}
unsafe impl cudarc::driver::DeviceRepr for GpuSphere {}
unsafe impl cudarc::driver::DeviceRepr for GpuMaterial {}
unsafe impl cudarc::driver::DeviceRepr for GpuTexture {}
unsafe impl cudarc::driver::DeviceRepr for GpuBvhNode {}

impl GpuCamera {
    pub fn new(
        image_width: u32,
        samples_per_pixel: u32,
        max_depth: u32,
        lookfrom: Point3,
        lookat: Point3,
        vup: Vec3,
        vfov_deg: f64,
        aspect_ratio: f64,
        aperture: f64,
        focus_dist: f64,
        background: Color,
    ) -> Self {
        let theta = vfov_deg.to_radians();
        let h = (theta / 2.0).tan();
        let viewport_height = 2.0 * h;
        let viewport_width = aspect_ratio * viewport_height;

        let w = vec3::unit_vector(lookfrom - lookat);
        let u = vec3::unit_vector(vec3::cross(vup, w));
        let v = vec3::cross(w, u);

        let origin = lookfrom;
        let horizontal = focus_dist * viewport_width * u;
        let vertical = focus_dist * viewport_height * v;
        let lower_left = origin - horizontal / 2.0 - vertical / 2.0 - focus_dist * w;

        let image_height = ((image_width as f64) / aspect_ratio) as u32;

        Self {
            origin: to_gpu(origin),
            lower_left_corner: to_gpu(lower_left),
            horizontal: to_gpu(horizontal),
            vertical: to_gpu(vertical),
            u: to_gpu(u),
            v: to_gpu(v),
            background: to_gpu(background),
            lens_radius: (aperture / 2.0) as f32,
            image_width,
            image_height,
            samples_per_pixel,
            max_depth,
            _pad: 0,
        }
    }
}

fn to_gpu(v: Vec3) -> GpuVec3 {
    GpuVec3 { x: v.x() as f32, y: v.y() as f32, z: v.z() as f32 }
}

fn add(a: GpuVec3, b: GpuVec3) -> GpuVec3 { GpuVec3 { x: a.x + b.x, y: a.y + b.y, z: a.z + b.z } }
fn sub(a: GpuVec3, b: GpuVec3) -> GpuVec3 { GpuVec3 { x: a.x - b.x, y: a.y - b.y, z: a.z - b.z } }
fn componentwise_min(a: GpuVec3, b: GpuVec3) -> GpuVec3 {
    GpuVec3 { x: a.x.min(b.x), y: a.y.min(b.y), z: a.z.min(b.z) }
}
fn componentwise_max(a: GpuVec3, b: GpuVec3) -> GpuVec3 {
    GpuVec3 { x: a.x.max(b.x), y: a.y.max(b.y), z: a.z.max(b.z) }
}

fn pad_aabb(mut bb: GpuAabb) -> GpuAabb {
    const DELTA: f32 = 0.0001;
    for axis in 0..3 {
        let (lo, hi) = match axis {
            0 => (&mut bb.min.x, &mut bb.max.x),
            1 => (&mut bb.min.y, &mut bb.max.y),
            _ => (&mut bb.min.z, &mut bb.max.z),
        };
        if *hi - *lo < DELTA {
            let half = DELTA * 0.5;
            *lo -= half;
            *hi += half;
        }
    }
    bb
}

fn sphere_aabb(s: &GpuSphere) -> GpuAabb {
    let r = GpuVec3 { x: s.radius, y: s.radius, z: s.radius };
    let a0 = GpuAabb { min: sub(s.center0, r), max: add(s.center0, r) };
    let bb = if s.is_moving == 0 {
        a0
    } else {
        let a1 = GpuAabb { min: sub(s.center1, r), max: add(s.center1, r) };
        GpuAabb {
            min: componentwise_min(a0.min, a1.min),
            max: componentwise_max(a0.max, a1.max),
        }
    };
    pad_aabb(bb)
}

fn aabb_union(a: GpuAabb, b: GpuAabb) -> GpuAabb {
    GpuAabb {
        min: componentwise_min(a.min, b.min),
        max: componentwise_max(a.max, b.max),
    }
}

fn longest_axis(bb: &GpuAabb) -> usize {
    let sx = bb.max.x - bb.min.x;
    let sy = bb.max.y - bb.min.y;
    let sz = bb.max.z - bb.min.z;
    if sx > sy { if sx > sz { 0 } else { 2 } }
    else { if sy > sz { 1 } else { 2 } }
}

fn axis_min(bb: &GpuAabb, axis: usize) -> f32 {
    match axis { 0 => bb.min.x, 1 => bb.min.y, _ => bb.min.z }
}

pub struct GpuScene {
    pub spheres: Vec<GpuSphere>,
    pub materials: Vec<GpuMaterial>,
    pub textures: Vec<GpuTexture>,
    pub bvh: Vec<GpuBvhNode>,
    pub bvh_root: u32,
}

pub struct GpuSceneBuilder {
    spheres: Vec<GpuSphere>,
    materials: Vec<GpuMaterial>,
    textures: Vec<GpuTexture>,
}

impl GpuSceneBuilder {
    pub fn new() -> Self {
        Self { spheres: Vec::new(), materials: Vec::new(), textures: Vec::new() }
    }

    pub fn add_solid_texture(&mut self, c: Color) -> u32 {
        let idx = self.textures.len() as u32;
        self.textures.push(GpuTexture {
            kind: TEX_SOLID,
            color: to_gpu(c),
            inv_scale: 0.0,
            tex_a: 0,
            tex_b: 0,
        });
        idx
    }

    pub fn add_checker_texture(&mut self, scale: f64, even: u32, odd: u32) -> u32 {
        // Depth-1 only: both `even` and `odd` must be SolidColor textures.
        assert!(self.textures[even as usize].kind == TEX_SOLID,
            "GPU CheckerTexture requires SolidColor children (depth-1)");
        assert!(self.textures[odd as usize].kind == TEX_SOLID,
            "GPU CheckerTexture requires SolidColor children (depth-1)");
        let idx = self.textures.len() as u32;
        self.textures.push(GpuTexture {
            kind: TEX_CHECKER,
            color: GpuVec3::default(),
            inv_scale: (1.0 / scale) as f32,
            tex_a: even,
            tex_b: odd,
        });
        idx
    }

    pub fn add_checker_colors(&mut self, scale: f64, c1: Color, c2: Color) -> u32 {
        let a = self.add_solid_texture(c1);
        let b = self.add_solid_texture(c2);
        self.add_checker_texture(scale, a, b)
    }

    pub fn add_lambertian(&mut self, tex_idx: u32) -> u32 {
        let idx = self.materials.len() as u32;
        self.materials.push(GpuMaterial {
            kind: MAT_LAMBERTIAN, tex_idx, fuzz: 0.0, ior: 0.0,
        });
        idx
    }

    pub fn add_lambertian_color(&mut self, albedo: Color) -> u32 {
        let t = self.add_solid_texture(albedo);
        self.add_lambertian(t)
    }

    pub fn add_metal(&mut self, albedo: Color, fuzz: f64) -> u32 {
        let t = self.add_solid_texture(albedo);
        let idx = self.materials.len() as u32;
        self.materials.push(GpuMaterial {
            kind: MAT_METAL,
            tex_idx: t,
            fuzz: fuzz.min(1.0) as f32,
            ior: 0.0,
        });
        idx
    }

    pub fn add_dielectric(&mut self, ior: f64) -> u32 {
        let idx = self.materials.len() as u32;
        self.materials.push(GpuMaterial {
            kind: MAT_DIELECTRIC, tex_idx: 0, fuzz: 0.0, ior: ior as f32,
        });
        idx
    }

    pub fn add_sphere(&mut self, c: Point3, r: f64, mat: u32) {
        self.spheres.push(GpuSphere {
            center0: to_gpu(c),
            center1: GpuVec3::default(),
            radius: r as f32,
            mat_idx: mat,
            is_moving: 0,
            _pad: 0,
        });
    }

    pub fn add_moving_sphere(&mut self, c0: Point3, c1: Point3, r: f64, mat: u32) {
        self.spheres.push(GpuSphere {
            center0: to_gpu(c0),
            center1: to_gpu(c1),
            radius: r as f32,
            mat_idx: mat,
            is_moving: 1,
            _pad: 0,
        });
    }

    pub fn build(self) -> GpuScene {
        let n = self.spheres.len();
        assert!(n > 0, "scene must contain at least one sphere");
        let mut indices: Vec<usize> = (0..n).collect();
        let mut nodes: Vec<GpuBvhNode> = Vec::new();
        let root = build_bvh(&self.spheres, &mut indices, 0, n, &mut nodes);
        GpuScene {
            spheres: self.spheres,
            materials: self.materials,
            textures: self.textures,
            bvh: nodes,
            bvh_root: root,
        }
    }
}

fn build_bvh(
    spheres: &[GpuSphere],
    indices: &mut [usize],
    start: usize,
    end: usize,
    nodes: &mut Vec<GpuBvhNode>,
) -> u32 {
    let span = end - start;
    let mut bbox = sphere_aabb(&spheres[indices[start]]);
    for i in (start + 1)..end {
        bbox = aabb_union(bbox, sphere_aabb(&spheres[indices[i]]));
    }

    if span == 1 {
        let idx = nodes.len() as u32;
        nodes.push(GpuBvhNode {
            bbox,
            left: 0,
            right: 0,
            leaf_idx: indices[start] as i32,
            _pad: 0,
        });
        return idx;
    }

    let axis = longest_axis(&bbox);
    indices[start..end].sort_by(|&a, &b| {
        let ca = axis_min(&sphere_aabb(&spheres[a]), axis);
        let cb = axis_min(&sphere_aabb(&spheres[b]), axis);
        ca.partial_cmp(&cb).unwrap_or(std::cmp::Ordering::Equal)
    });
    let mid = start + span / 2;
    let left = build_bvh(spheres, indices, start, mid, nodes);
    let right = build_bvh(spheres, indices, mid, end, nodes);
    let merged = aabb_union(nodes[left as usize].bbox, nodes[right as usize].bbox);
    let idx = nodes.len() as u32;
    nodes.push(GpuBvhNode {
        bbox: merged,
        left,
        right,
        leaf_idx: -1,
        _pad: 0,
    });
    idx
}
