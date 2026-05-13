use crate::color::Color;
use crate::hittable::HitRecord;
use crate::ray::Ray;
use crate::vec3::Point3;
use crate::{common, vec3};
use crate::texture::Texture;
use crate::texture::SolidColor;
use std::rc::Rc;


pub struct ScatterRecord {
    pub attenuation: Color,
    pub scattered: Ray,
}

pub trait Material {
    fn scatter(&self, r_in: &Ray, rec: &HitRecord) -> Option<ScatterRecord>;
    fn emitted(&self, _u: f64, _v: f64, _p: &Point3) -> Color {
        Color::new(0.0, 0.0, 0.0)
    }
}

fn reflectance(cosine: f64, ref_idx: f64) -> f64 {
    // Use Schlick's approximation for reflectance
    let mut r0 = (1.0 - ref_idx) / (1.0 + ref_idx);
    r0 = r0 * r0;
    r0 + (1.0 - r0) * f64::powf(1.0 - cosine, 5.0)
}

pub struct Lambertian {
    tex: Rc<dyn Texture>,
}

impl Lambertian {
    pub fn new(albedo: Color) -> Self {
        Self { tex: Rc::new(SolidColor::new(albedo)) }
    }
 
    pub fn from_texture(tex: Rc<dyn Texture>) -> Self {
        Self { tex }
    }
}

impl Material for Lambertian {
    fn scatter(&self, r_in: &Ray, rec: &HitRecord) -> Option<ScatterRecord> {
        let mut scatter_direction = rec.normal + vec3::random_unit_vector();
 
         // Catch degenerate scatter direction
        if scatter_direction.near_zero() {
            scatter_direction = rec.normal;
        }

        Some(ScatterRecord {
            attenuation: self.tex.value(rec.u, rec.v, &rec.p),
            scattered: Ray::new(rec.p, scatter_direction, r_in.time()),
        })
    }
}

pub struct Metal {
    albedo: Color,
    fuzz: f64,
}
 
impl Metal {
    pub fn new(a: Color, f: f64) -> Metal {
        Metal {
            albedo: a,
            fuzz: if f < 1.0 { f } else { 1.0 },
        }
    }
}
 
impl Material for Metal {
    fn scatter(&self, r_in: &Ray, rec: &HitRecord) -> Option<ScatterRecord> {
        let reflected = vec3::reflect(vec3::unit_vector(r_in.direction()), rec.normal);
        let scattered = Ray::new(rec.p, reflected + self.fuzz * vec3::random_in_unit_sphere(), r_in.time());
 
        if vec3::dot(scattered.direction(), rec.normal) > 0.0 {
            Some(ScatterRecord {
                attenuation: self.albedo,
                scattered,
            })
        } else {
            None
        }
    }
}


pub struct Dielectric {
    ir: f64, // Index of refraction
}
 
impl Dielectric {
    pub fn new(index_of_refraction: f64) -> Dielectric {
        Dielectric {
            ir: index_of_refraction,
        }
    }
}
 
impl Material for Dielectric {
    fn scatter(&self, r_in: &Ray, rec: &HitRecord) -> Option<ScatterRecord> {
        let refraction_ratio = if rec.front_face {
            1.0 / self.ir
        } else {
            self.ir
        };
 
        let unit_direction = vec3::unit_vector(r_in.direction());
        let cos_theta = f64::min(vec3::dot(-unit_direction, rec.normal), 1.0);
        let sin_theta = f64::sqrt(1.0 - cos_theta * cos_theta);
 
        let cannot_refract = refraction_ratio * sin_theta > 1.0;
        let direction = if cannot_refract 
        || reflectance(cos_theta, refraction_ratio) > common::random_double() {
            vec3::reflect(unit_direction, rec.normal)
        } else {
            vec3::refract(unit_direction, rec.normal, refraction_ratio)
        };
 
        Some(ScatterRecord {
            attenuation: Color::new(1.0, 1.0, 1.0),
            scattered: Ray::new(rec.p, direction, r_in.time()),
        })
    }
}

pub struct DiffuseLight {
    emit: Rc<dyn Texture>,
}
 
impl DiffuseLight {
    pub fn new(emit: Rc<dyn Texture>) -> Self {
        Self { emit }
    }
 
    pub fn from_color(color: Color) -> Self {
        Self { emit: Rc::new(SolidColor::new(color)) }
    }
}
 
impl Material for DiffuseLight {
    fn scatter(&self, r_in: &Ray, rec: &HitRecord) -> Option<ScatterRecord> {
        None
    }
 
    fn emitted(&self, u: f64, v: f64, p: &Point3) -> Color {
        self.emit.value(u, v, p)
    }
}

pub struct Isotropic {
    tex: Rc<dyn Texture>,
}
 
impl Isotropic {
    pub fn new(tex: Rc<dyn Texture>) -> Self {
        Self { tex }
    }
 
    pub fn from_color(albedo: Color) -> Self {
        Self { tex: Rc::new(SolidColor::new(albedo)) }
    }
}
 
impl Material for Isotropic {
    fn scatter(&self, r_in: &Ray, rec: &HitRecord) -> Option<ScatterRecord> {
        Some(ScatterRecord {
            attenuation: self.tex.value(rec.u, rec.v, &rec.p),
            scattered: Ray::new(rec.p, vec3::random_unit_vector(), r_in.time()),
        })
    }
}