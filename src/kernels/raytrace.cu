// CUDA kernel mirroring the CPU ray tracer in f32.
// Scoped to: Sphere primitives, Lambertian/Metal/Dielectric materials,
// SolidColor / depth-1 CheckerTexture, motion blur, depth of field, BVH traversal.
//
// Layouts must byte-match src/gpu/scene.rs structs.

#include <stdint.h>

// ------------------------------------------------------------ Math primitives

struct Vec3 {
    float x, y, z;
};

__device__ inline Vec3 v3(float x, float y, float z) { Vec3 r; r.x = x; r.y = y; r.z = z; return r; }
__device__ inline Vec3 operator+(Vec3 a, Vec3 b) { return v3(a.x+b.x, a.y+b.y, a.z+b.z); }
__device__ inline Vec3 operator-(Vec3 a, Vec3 b) { return v3(a.x-b.x, a.y-b.y, a.z-b.z); }
__device__ inline Vec3 operator-(Vec3 a) { return v3(-a.x, -a.y, -a.z); }
__device__ inline Vec3 operator*(Vec3 a, Vec3 b) { return v3(a.x*b.x, a.y*b.y, a.z*b.z); }
__device__ inline Vec3 operator*(float t, Vec3 a) { return v3(t*a.x, t*a.y, t*a.z); }
__device__ inline Vec3 operator*(Vec3 a, float t) { return v3(t*a.x, t*a.y, t*a.z); }
__device__ inline Vec3 operator/(Vec3 a, float t) { float k = 1.0f/t; return v3(a.x*k, a.y*k, a.z*k); }
__device__ inline float dot(Vec3 a, Vec3 b) { return a.x*b.x + a.y*b.y + a.z*b.z; }
__device__ inline Vec3 cross(Vec3 a, Vec3 b) {
    return v3(a.y*b.z - a.z*b.y, a.z*b.x - a.x*b.z, a.x*b.y - a.y*b.x);
}
__device__ inline float length_sq(Vec3 a) { return dot(a, a); }
__device__ inline float length(Vec3 a) { return sqrtf(length_sq(a)); }
__device__ inline Vec3 unit(Vec3 a) { return a / length(a); }
__device__ inline bool near_zero(Vec3 a) {
    const float E = 1e-8f;
    return fabsf(a.x) < E && fabsf(a.y) < E && fabsf(a.z) < E;
}
__device__ inline Vec3 reflect(Vec3 v, Vec3 n) { return v - 2.0f * dot(v, n) * n; }
__device__ inline Vec3 refract(Vec3 uv, Vec3 n, float etai_over_etat) {
    float cos_theta = fminf(dot(-uv, n), 1.0f);
    Vec3 r_perp = etai_over_etat * (uv + cos_theta * n);
    Vec3 r_par  = -sqrtf(fabsf(1.0f - length_sq(r_perp))) * n;
    return r_perp + r_par;
}

// ------------------------------------------------------------ RNG (xorshift32)

struct Rng {
    uint32_t state;
};
__device__ inline uint32_t xor32(Rng& r) {
    uint32_t x = r.state;
    x ^= x << 13; x ^= x >> 17; x ^= x << 5;
    r.state = x ? x : 0x9E3779B9u;
    return r.state;
}
__device__ inline float rand_f(Rng& r) {
    // [0, 1)
    return (xor32(r) >> 8) * (1.0f / 16777216.0f);
}
__device__ inline float rand_range(Rng& r, float lo, float hi) { return lo + (hi - lo) * rand_f(r); }
__device__ inline Vec3 rand_in_unit_sphere(Rng& r) {
    for (int i = 0; i < 32; ++i) {
        Vec3 p = v3(rand_range(r,-1,1), rand_range(r,-1,1), rand_range(r,-1,1));
        if (length_sq(p) < 1.0f) return p;
    }
    return v3(0,0,0);
}
__device__ inline Vec3 rand_unit_vec(Rng& r) { return unit(rand_in_unit_sphere(r)); }
__device__ inline Vec3 rand_in_unit_disk(Rng& r) {
    for (int i = 0; i < 32; ++i) {
        Vec3 p = v3(rand_range(r,-1,1), rand_range(r,-1,1), 0.0f);
        if (length_sq(p) < 1.0f) return p;
    }
    return v3(0,0,0);
}

// ------------------------------------------------------------ Scene types
// Must byte-match src/gpu/scene.rs.

struct Aabb { Vec3 min; Vec3 max; };

struct Sphere {
    Vec3 center0;
    Vec3 center1;
    float radius;
    uint32_t mat_idx;
    uint32_t is_moving;
    uint32_t _pad;
};

struct Material {
    uint32_t kind;       // 0=Lambertian, 1=Metal, 2=Dielectric
    uint32_t tex_idx;
    float fuzz;
    float ior;
};

struct Texture {
    uint32_t kind;       // 0=SolidColor, 1=Checker
    Vec3 color;
    float inv_scale;
    uint32_t tex_a;
    uint32_t tex_b;
};

struct BvhNode {
    Aabb bbox;
    uint32_t left;
    uint32_t right;
    int32_t  leaf_idx;
    uint32_t _pad;
};

struct Camera {
    Vec3 origin;
    Vec3 lower_left_corner;
    Vec3 horizontal;
    Vec3 vertical;
    Vec3 u;
    Vec3 v;
    Vec3 background;
    float lens_radius;
    uint32_t image_width;
    uint32_t image_height;
    uint32_t samples_per_pixel;
    uint32_t max_depth;
    uint32_t _pad;
};

// Static size checks to catch host/device layout drift early.
static_assert(sizeof(Vec3) == 12, "Vec3 size");
static_assert(sizeof(Aabb) == 24, "Aabb size");
static_assert(sizeof(Sphere) == 40, "Sphere size");
static_assert(sizeof(Material) == 16, "Material size");
static_assert(sizeof(Texture) == 28, "Texture size");
static_assert(sizeof(BvhNode) == 40, "BvhNode size");
static_assert(sizeof(Camera) == 108, "Camera size");

// ------------------------------------------------------------ Texture eval

__device__ Vec3 texture_value_solid(const Texture& t) { return t.color; }

__device__ Vec3 texture_value(const Texture* textures, uint32_t idx, float, float, Vec3 p) {
    const Texture t = textures[idx];
    if (t.kind == 0) return t.color;
    // CheckerTexture: depth-1 (children must be SolidColor, enforced on host).
    int xi = (int)floorf(t.inv_scale * p.x);
    int yi = (int)floorf(t.inv_scale * p.y);
    int zi = (int)floorf(t.inv_scale * p.z);
    bool even = ((xi + yi + zi) & 1) == 0;
    uint32_t child = even ? t.tex_a : t.tex_b;
    return textures[child].color;  // child guaranteed SolidColor
}

// ------------------------------------------------------------ Ray / Hit

struct Ray { Vec3 orig; Vec3 dir; float tm; };

__device__ inline Vec3 ray_at(const Ray& r, float t) { return r.orig + t * r.dir; }

struct HitRecord {
    Vec3 p;
    Vec3 normal;
    uint32_t mat_idx;
    float t;
    float u;
    float v;
    bool front_face;
};

__device__ inline void set_face_normal(HitRecord& rec, const Ray& r, Vec3 outward) {
    rec.front_face = dot(r.dir, outward) < 0.0f;
    rec.normal = rec.front_face ? outward : -outward;
}

// ------------------------------------------------------------ Sphere & AABB hit

__device__ inline Vec3 sphere_center_at(const Sphere& s, float time) {
    if (s.is_moving == 0) return s.center0;
    return s.center0 + time * (s.center1 - s.center0);
}

__device__ bool sphere_hit(const Sphere& s, const Ray& r, float t_min, float t_max, HitRecord& rec) {
    Vec3 center = sphere_center_at(s, r.tm);
    Vec3 oc = r.orig - center;
    float a = length_sq(r.dir);
    float half_b = dot(oc, r.dir);
    float c = length_sq(oc) - s.radius * s.radius;
    float disc = half_b * half_b - a * c;
    if (disc < 0.0f) return false;
    float sqrt_d = sqrtf(disc);

    float root = (-half_b - sqrt_d) / a;
    if (root <= t_min || t_max <= root) {
        root = (-half_b + sqrt_d) / a;
        if (root <= t_min || t_max <= root) return false;
    }
    rec.t = root;
    rec.p = ray_at(r, root);
    rec.mat_idx = s.mat_idx;
    Vec3 outward = (rec.p - center) / s.radius;
    set_face_normal(rec, r, outward);
    rec.u = 0.0f; rec.v = 0.0f;  // unused for the supported materials
    return true;
}

__device__ bool aabb_hit(const Aabb& bb, const Ray& r, float t_min, float t_max) {
    #pragma unroll
    for (int a = 0; a < 3; ++a) {
        float orig = (a == 0) ? r.orig.x : (a == 1) ? r.orig.y : r.orig.z;
        float dir  = (a == 0) ? r.dir.x  : (a == 1) ? r.dir.y  : r.dir.z;
        float mn   = (a == 0) ? bb.min.x : (a == 1) ? bb.min.y : bb.min.z;
        float mx   = (a == 0) ? bb.max.x : (a == 1) ? bb.max.y : bb.max.z;
        float inv = 1.0f / dir;
        float t0 = (mn - orig) * inv;
        float t1 = (mx - orig) * inv;
        if (inv < 0.0f) { float tmp = t0; t0 = t1; t1 = tmp; }
        if (t0 > t_min) t_min = t0;
        if (t1 < t_max) t_max = t1;
        if (t_max <= t_min) return false;
    }
    return true;
}

// ------------------------------------------------------------ BVH traversal (iterative)

__device__ bool bvh_hit(
    const BvhNode* bvh, uint32_t root,
    const Sphere* spheres,
    const Ray& r, float t_min, float t_max,
    HitRecord& best)
{
    uint32_t stack[64];
    int sp = 0;
    stack[sp++] = root;
    float closest = t_max;
    bool hit_anything = false;

    while (sp > 0) {
        uint32_t idx = stack[--sp];
        const BvhNode node = bvh[idx];
        if (!aabb_hit(node.bbox, r, t_min, closest)) continue;
        if (node.leaf_idx >= 0) {
            HitRecord tmp;
            if (sphere_hit(spheres[node.leaf_idx], r, t_min, closest, tmp)) {
                hit_anything = true;
                closest = tmp.t;
                best = tmp;
            }
        } else {
            if (sp + 2 <= 64) {
                stack[sp++] = node.left;
                stack[sp++] = node.right;
            }
        }
    }
    return hit_anything;
}

// ------------------------------------------------------------ Material scatter

__device__ inline float reflectance(float cosine, float ref_idx) {
    float r0 = (1.0f - ref_idx) / (1.0f + ref_idx);
    r0 = r0 * r0;
    return r0 + (1.0f - r0) * powf(1.0f - cosine, 5.0f);
}

__device__ bool material_scatter(
    const Material& m,
    const Texture* textures,
    const Ray& r_in,
    const HitRecord& rec,
    Rng& rng,
    Vec3& attenuation,
    Ray& scattered)
{
    if (m.kind == 0) {
        // Lambertian
        Vec3 dir = rec.normal + rand_unit_vec(rng);
        if (near_zero(dir)) dir = rec.normal;
        attenuation = texture_value(textures, m.tex_idx, rec.u, rec.v, rec.p);
        scattered.orig = rec.p; scattered.dir = dir; scattered.tm = r_in.tm;
        return true;
    }
    if (m.kind == 1) {
        // Metal
        Vec3 reflected = reflect(unit(r_in.dir), rec.normal);
        Vec3 fuzzed = reflected + m.fuzz * rand_in_unit_sphere(rng);
        if (dot(fuzzed, rec.normal) <= 0.0f) return false;
        attenuation = textures[m.tex_idx].color;
        scattered.orig = rec.p; scattered.dir = fuzzed; scattered.tm = r_in.tm;
        return true;
    }
    // Dielectric (kind == 2)
    float refraction_ratio = rec.front_face ? (1.0f / m.ior) : m.ior;
    Vec3 unit_dir = unit(r_in.dir);
    float cos_theta = fminf(dot(-unit_dir, rec.normal), 1.0f);
    float sin_theta = sqrtf(1.0f - cos_theta * cos_theta);
    bool cannot_refract = refraction_ratio * sin_theta > 1.0f;
    Vec3 dir;
    if (cannot_refract || reflectance(cos_theta, refraction_ratio) > rand_f(rng)) {
        dir = reflect(unit_dir, rec.normal);
    } else {
        dir = refract(unit_dir, rec.normal, refraction_ratio);
    }
    attenuation = v3(1.0f, 1.0f, 1.0f);
    scattered.orig = rec.p; scattered.dir = dir; scattered.tm = r_in.tm;
    return true;
}

// ------------------------------------------------------------ Camera

__device__ Ray camera_get_ray(const Camera& c, float s, float t, Rng& rng) {
    Vec3 rd = c.lens_radius * rand_in_unit_disk(rng);
    Vec3 offset = c.u * rd.x + c.v * rd.y;
    Ray r;
    r.orig = c.origin + offset;
    r.dir  = c.lower_left_corner + s * c.horizontal + t * c.vertical - c.origin - offset;
    r.tm   = rand_f(rng);
    return r;
}

// ------------------------------------------------------------ Ray color (iterative)

__device__ Vec3 ray_color(
    Ray r,
    Vec3 background,
    const BvhNode* bvh, uint32_t bvh_root,
    const Sphere* spheres,
    const Material* materials,
    const Texture* textures,
    int max_depth,
    Rng& rng)
{
    Vec3 attenuation = v3(1.0f, 1.0f, 1.0f);
    Vec3 color = v3(0.0f, 0.0f, 0.0f);

    for (int depth = 0; depth < max_depth; ++depth) {
        HitRecord rec;
        if (!bvh_hit(bvh, bvh_root, spheres, r, 0.001f, 1.0e30f, rec)) {
            color = color + attenuation * background;
            return color;
        }
        Material m = materials[rec.mat_idx];
        Vec3 atten_step; Ray scattered;
        if (!material_scatter(m, textures, r, rec, rng, atten_step, scattered)) {
            return color;  // emission already accounted for (zero for non-light materials in scope)
        }
        attenuation = attenuation * atten_step;
        r = scattered;
    }
    return color;
}

// ------------------------------------------------------------ Kernel entry

extern "C" __global__ void render(
    unsigned char* out_rgb,
    const Sphere* spheres, uint32_t n_spheres,
    const Material* materials,
    const Texture* textures,
    const BvhNode* bvh, uint32_t bvh_root,
    Camera cam,
    unsigned long long seed)
{
    uint32_t i = blockIdx.x * blockDim.x + threadIdx.x;
    uint32_t j = blockIdx.y * blockDim.y + threadIdx.y;
    if (i >= cam.image_width || j >= cam.image_height) return;

    // Seed per-pixel RNG. Mix pixel coords into the global seed.
    uint64_t s = seed ^ ((uint64_t)i * 0x9E3779B97F4A7C15ull) ^ ((uint64_t)j * 0xBF58476D1CE4E5B9ull);
    Rng rng;
    rng.state = (uint32_t)(s ^ (s >> 32));
    if (rng.state == 0) rng.state = 0x12345678u;

    Vec3 pixel_color = v3(0.0f, 0.0f, 0.0f);
    for (uint32_t k = 0; k < cam.samples_per_pixel; ++k) {
        float u = ((float)i + rand_f(rng)) / (float)(cam.image_width  - 1);
        float v = ((float)j + rand_f(rng)) / (float)(cam.image_height - 1);
        Ray r = camera_get_ray(cam, u, v, rng);
        pixel_color = pixel_color + ray_color(r, cam.background, bvh, bvh_root, spheres, materials, textures, (int)cam.max_depth, rng);
    }

    // Gamma-2 + clamp + 8-bit quantize (matches color::write_color in Rust).
    float scale = 1.0f / (float)cam.samples_per_pixel;
    float r = sqrtf(scale * pixel_color.x);
    float g = sqrtf(scale * pixel_color.y);
    float b = sqrtf(scale * pixel_color.z);
    if (r < 0.0f) r = 0.0f; if (r > 0.999f) r = 0.999f;
    if (g < 0.0f) g = 0.0f; if (g > 0.999f) g = 0.999f;
    if (b < 0.0f) b = 0.0f; if (b > 0.999f) b = 0.999f;

    uint32_t idx = (j * cam.image_width + i) * 3;
    out_rgb[idx + 0] = (unsigned char)(256.0f * r);
    out_rgb[idx + 1] = (unsigned char)(256.0f * g);
    out_rgb[idx + 2] = (unsigned char)(256.0f * b);

    // Suppress unused-warning for n_spheres (BVH covers traversal; n_spheres kept for ABI symmetry).
    (void)n_spheres;
}
