use image::RgbImage;

use crate::color::Color;
use crate::vec3::Point3;
use std::rc::Rc;
use crate::perlin;

pub trait Texture {
    fn value(&self, u: f64, v: f64, p: &Point3) -> Color;
}
 
pub struct SolidColor {
    albedo: Color,
}
 
impl SolidColor {
    pub fn new(albedo: Color) -> Self {
        Self { albedo }
    }
 
    pub fn from_rgb(red: f64, green: f64, blue: f64) -> Self {
        Self { albedo: Color::new(red, green, blue) }
    }
}
 
impl Texture for SolidColor {
    fn value(&self, _u: f64, _v: f64, _p: &Point3) -> Color {
        self.albedo
    }
}

pub struct CheckerTexture {
    inv_scale: f64,
    even: Rc<dyn Texture>,
    odd: Rc<dyn Texture>,
}

impl CheckerTexture {
    pub fn new(scale: f64, even: Rc<dyn Texture>, odd: Rc<dyn Texture>) -> Self {
        Self { inv_scale: 1.0 / scale, even, odd }
    }
 
    pub fn from_colors(scale: f64, c1: Color, c2: Color) -> Self {
        Self::new(
            scale,
            Rc::new(SolidColor::new(c1)),
            Rc::new(SolidColor::new(c2)),
        )
    }
}

impl Texture for CheckerTexture {
    fn value(&self, u: f64, v: f64, p: &Point3) -> Color {
        let x = (self.inv_scale * p.x()).floor() as i32;
        let y = (self.inv_scale * p.y()).floor() as i32;
        let z = (self.inv_scale * p.z()).floor() as i32;
        if (x + y + z) % 2 == 0 {
            self.even.value(u, v, p)
        } else {
            self.odd.value(u, v, p)
        }
    }
}

pub struct ImageTexture {
    image: RgbImage,
}

impl ImageTexture {
    pub fn new(filename: &str) -> Self {
        let image = image::open(filename)
            .expect("Could not open image file")
            .into_rgb8();
        Self { image }
    }
}

impl Texture for ImageTexture {
    fn value(&self, u: f64, v: f64, _p: &Point3) -> Color {
        if self.image.height() == 0 {
            return Color::new(0.0, 1.0, 1.0);
        }
        let u = u.clamp(0.0, 1.0);
        let v = 1.0 - v.clamp(0.0, 1.0);
        let i = (u * self.image.width() as f64) as u32;
        let j = (v * self.image.height() as f64) as u32;
        let i = i.min(self.image.width() - 1);
        let j = j.min(self.image.height() - 1);
        let pixel = self.image.get_pixel(i, j);
        let scale = 1.0 / 255.0;
        Color::new(
            scale * pixel[0] as f64,
            scale * pixel[1] as f64,
            scale * pixel[2] as f64,
        )
    }
}

pub struct NoiseTexture {
    noise: perlin::Perlin,
    scale: f64,
}
 
impl NoiseTexture {
    pub fn new(scale: f64) -> Self {
        Self { noise: perlin::Perlin::new(), scale }
    }
}
 
impl Texture for NoiseTexture {
    fn value(&self, _u: f64, _v: f64, p: &Point3) -> Color {
        let noise_val = 1.0 + (self.scale * p.z() + 10.0 * self.noise.turbulence(p, 7)).sin();
        Color::new(1.0, 1.0, 1.0) * 0.5 * noise_val
    }
}