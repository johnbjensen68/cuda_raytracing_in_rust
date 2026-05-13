use crate::color::Color;
use crate::common;
use crate::color;
use crate::hittable::Hittable;
use crate::ray::Ray;
use crate::vec3::{self, Point3, Vec3};
 use std::io;

pub struct Camera {
    aspect_ratio: f64,
    image_width: usize,
    samples_per_pixel: i32,
    max_depth: i32,
    origin: Point3,
    lower_left_corner: Point3,
    horizontal: Vec3,
    vertical: Vec3,
    u: Vec3,
    v: Vec3,
    lens_radius: f64,
    pub background: Color,
}
 
impl Camera {
    pub fn new(
        image_width: usize,
        samples_per_pixel: i32,
        max_depth: i32,
        lookfrom: Point3,
        lookat: Point3,
        vup: Vec3,
        vfov: f64, // Vertical field-of-view in degrees
        aspect_ratio: f64,
        aperture: f64,
        focus_dist: f64,
        background: Color
    ) -> Camera {
        let theta = common::degrees_to_radians(vfov);
        let h = f64::tan(theta / 2.0);
        let viewport_height = 2.0 * h;
        let viewport_width = aspect_ratio * viewport_height;
 
        let w = vec3::unit_vector(lookfrom - lookat);
        let u = vec3::unit_vector(vec3::cross(vup, w));
        let v = vec3::cross(w, u);
 
        let origin = lookfrom;
        let horizontal = focus_dist * viewport_width * u;
        let vertical = focus_dist * viewport_height * v;
        let lower_left_corner = origin - horizontal / 2.0 - vertical / 2.0 - focus_dist * w;
 
        let lens_radius = aperture / 2.0;
 
        Camera {
            aspect_ratio,
            image_width,
            samples_per_pixel,
            max_depth,
            origin,
            lower_left_corner,
            horizontal,
            vertical,
            u,
            v,
            lens_radius,
            background
        }
    }
 
    fn ray_color(&self, r: &Ray, background: Color, world: &dyn Hittable, depth: i32) -> Color {
        if depth <= 0 {
            return Color::new(0.0, 0.0, 0.0);
        }
        let Some(rec) = world.hit(r, 0.001, f64::INFINITY) else {
            return background;
        };
        let color_from_emission = rec.mat.emitted(rec.u, rec.v, &rec.p);

        let Some(scatter_rec) = rec.mat.scatter(r, &rec) else  {
            return color_from_emission;
        };
        let color_from_scatter =
            scatter_rec.attenuation * self.ray_color(&scatter_rec.scattered, background, world, depth - 1);
        color_from_emission + color_from_scatter
    }
 
    pub fn get_ray(&self, s: f64, t: f64) -> Ray {
        let rd = self.lens_radius * vec3::random_in_unit_disk();
        let offset = self.u * rd.x() + self.v * rd.y();
        let ray_time = common::random_double();

        Ray::new(
            self.origin + offset,
            self.lower_left_corner + s * self.horizontal + t * self.vertical - self.origin - offset,
            ray_time
        )
    }

    pub fn render(&self, world: &dyn Hittable) {
        let image_height: i32 = (self.image_width as f64 / self.aspect_ratio) as i32;


        print!("P3\n{} {}\n255\n", self.image_width, image_height);

        for j in (0..image_height).rev() {
            eprint!("\rScanlines remaining: {} ", j);
            for i in 0..self.image_width {
                let mut pixel_color = Color::new(0.0, 0.0, 0.0);

                for _ in 0..self.samples_per_pixel {
                    let u = (i as f64 + common::random_double()) / (self.image_width - 1) as f64;
                    let v = (j as f64 + common::random_double()) / (image_height - 1) as f64;
                    let r = self.get_ray(u, v);
                    pixel_color += self.ray_color(&r, self.background, world, self.max_depth);
                }
                color::write_color(&mut io::stdout(), pixel_color, self.samples_per_pixel);
            }
        }
        eprint!("\nDone.\n");
    }
}