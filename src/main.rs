mod color;
mod common;
mod hittable;
mod hittable_list;
mod vec3;
mod ray;
mod sphere;
mod camera;
mod material;
mod interval;
mod aabb;
mod bvh;
mod texture;
mod perlin;
mod quad;
mod constant_medium;

use std::rc::Rc;
use color::Color;
use hittable_list::HittableList;
use material::{Lambertian, Metal, Dielectric};
use sphere::Sphere;
use vec3::{Point3};
use bvh::BvhNode;
use camera::Camera;
use texture::CheckerTexture;
use texture::NoiseTexture;
use crate::constant_medium::ConstantMedium;
use crate::hittable::Hittable;
use crate::hittable::RotateY;
use crate::hittable::Translate;
use crate::material::DiffuseLight;
use crate::quad::Quad;
use crate::{texture::ImageTexture, vec3::Vec3};



fn random_scene() -> HittableList {
    let mut world = HittableList::new();
 
     let checker = Rc::new(CheckerTexture::from_colors(
        0.32,
        Color::new(0.2, 0.3, 0.1),
        Color::new(0.9, 0.9, 0.9),
    ));
    let sp1 = Sphere::new(
        Point3::new(0.0, -1000.0, 0.0),
        1000.0,
        Rc::new(Lambertian::from_texture(checker)),
    );
    world.add(Box::new(sp1));

    let ground_material = Rc::new(Lambertian::new(Color::new(0.5, 0.5, 0.5)));
    world.add(Box::new(Sphere::new(
        Point3::new(0.0, -1000.0, 0.0),
        1000.0,
        ground_material,
    )));
 
    for a in -11..11 {
        for b in -11..11 {
            let choose_mat = common::random_double();
            let center = Point3::new(
                a as f64 + 0.9 * common::random_double(),
                0.2,
                b as f64 + 0.9 * common::random_double(),
            );
 
            if (center - Point3::new(4.0, 0.2, 0.0)).length() > 0.9 {
                if choose_mat < 0.8 {
                    // Diffuse
                    let albedo = Color::random() * Color::random();
                    let sphere_material = Rc::new(Lambertian::new(albedo));
                    let center2 = center + vec3::Vec3::new(0.0, common::random_double_range(0.0,0.5), 0.0);
                    world.add(Box::new(Sphere::new_moving(center, center2, 0.2, sphere_material)));
                } else if choose_mat < 0.95 {
                    // Metal
                    let albedo = Color::random_range(0.5, 1.0);
                    let fuzz = common::random_double_range(0.0, 0.5);
                    let sphere_material = Rc::new(Metal::new(albedo, fuzz));
                    world.add(Box::new(Sphere::new(center, 0.2, sphere_material)));
                } else {
                    // Glass
                    let sphere_material = Rc::new(Dielectric::new(1.5));
                    world.add(Box::new(Sphere::new(center, 0.2, sphere_material)));
                }
            }
        }
    }
 
    let material1 = Rc::new(Dielectric::new(1.5));
    world.add(Box::new(Sphere::new(
        Point3::new(0.0, 1.0, 0.0),
        1.0,
        material1,
    )));
 
    let material2 = Rc::new(Lambertian::new(Color::new(0.4, 0.2, 0.1)));
    world.add(Box::new(Sphere::new(
        Point3::new(-4.0, 1.0, 0.0),
        1.0,
        material2,
    )));
 
    let material3 = Rc::new(Metal::new(Color::new(0.7, 0.6, 0.5), 0.0));
    world.add(Box::new(Sphere::new(
        Point3::new(4.0, 1.0, 0.0),
        1.0,
        material3,
    )));
 
    world
}

fn bouncing_spheres() {
    const ASPECT_RATIO: f64 = 3.0 / 2.0;

    // World
    let world = BvhNode::new(random_scene());
    //let world = random_scene();

    let lookfrom = Point3::new(13.0, 2.0, 3.0);
    let lookat = Point3::new(0.0, 0.0, 0.0);
    let vup = Point3::new(0.0, 1.0, 0.0);
    let dist_to_focus = 10.0;
    let aperture = 0.1;

    const IMAGE_WIDTH: usize = 400;
    const SAMPLES_PER_PIXEL: i32 = 100;
    const MAX_DEPTH: i32 = 50;

    let cam = Camera::new(
        IMAGE_WIDTH,
        SAMPLES_PER_PIXEL,
        MAX_DEPTH,
        lookfrom,
        lookat,
        vup,
        20.0,
        ASPECT_RATIO,
        aperture,
        dist_to_focus,
        Color::new(0.70, 0.80, 1.00),
    );

    cam.render(&world);
}

fn checkered_spheres() {
    let mut world = HittableList::new();
 
    let checker = Rc::new(CheckerTexture::from_colors(
        0.32,
        Color::new(0.2, 0.3, 0.1),
        Color::new(0.9, 0.9, 0.9),
    ));
    world.add(Box::new(Sphere::new(
        Point3::new(0.0, -10.0, 0.0), 10.0,
        Rc::new(Lambertian::from_texture(checker.clone())),
    )));
    world.add(Box::new(Sphere::new(
        Point3::new(0.0, 10.0, 0.0), 10.0,
        Rc::new(Lambertian::from_texture(checker)),
    )));
 
    const IMAGE_WIDTH: usize = 400;
    const SAMPLES_PER_PIXEL: i32 = 100;
    const MAX_DEPTH: i32 = 50;
    let cam = Camera::new(
        IMAGE_WIDTH,
        SAMPLES_PER_PIXEL,
        MAX_DEPTH,
        Point3::new(13.0, 2.0, 3.0),
        Point3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
        20.0,
        16.0 / 9.0,
        0.1,
        10.0,
        Color::new(0.70, 0.80, 1.00)
    );

    cam.render(&world);
}

fn earth() {
    let earth_texture = Rc::new(ImageTexture::new("earthmap.jpg"));
    let earth_surface = Rc::new(Lambertian::from_texture(earth_texture));
    let globe = Box::new(Sphere::new(
        Point3::new(0.0, 0.0, 0.0), 2.0, earth_surface,
    ));
     let mut world = HittableList::new();
     world.add(globe);

    const IMAGE_WIDTH: usize = 400;
    const SAMPLES_PER_PIXEL: i32 = 100;
    const MAX_DEPTH: i32 = 50;
     let cam = Camera::new(        
        IMAGE_WIDTH,
        SAMPLES_PER_PIXEL,
        MAX_DEPTH,
        Point3::new(0.0, 0.0, 12.0),
        Point3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
        20.0,
        16.0 / 9.0,
        0.1,
        10.0,
        Color::new(0.70, 0.80, 1.00)
    );

    cam.render(&world);
}

fn perlin_spheres() {
    let mut world = HittableList::new();
 
    let pertext = Rc::new(NoiseTexture::new(4.0));
    world.add(Box::new(Sphere::new(
        Point3::new(0.0, -1000.0, 0.0), 1000.0,
        Rc::new(Lambertian::from_texture(pertext.clone())),
    )));
    world.add(Box::new(Sphere::new(
        Point3::new(0.0, 2.0, 0.0), 2.0,
        Rc::new(Lambertian::from_texture(pertext)),
    )));

    let cam = Camera::new(
        400,
        100,
        50,
        Point3::new(13.0, 2.0, 3.0),
        Point3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
        20.0,
        16.0 / 9.0,
        0.1,
        10.0,
        Color::new(0.70, 0.80, 1.00));

    cam.render(&world);
}


fn quads() {
    let mut world = HittableList::new();
 
    // Materials
    let left_red     = Rc::new(Lambertian::new(Color::new(1.0, 0.2, 0.2)));
    let back_green   = Rc::new(Lambertian::new(Color::new(0.2, 1.0, 0.2)));
    let right_blue   = Rc::new(Lambertian::new(Color::new(0.2, 0.2, 1.0)));
    let upper_orange = Rc::new(Lambertian::new(Color::new(1.0, 0.5, 0.0)));
    let lower_teal   = Rc::new(Lambertian::new(Color::new(0.2, 0.8, 0.8)));
 
    // Quads
    world.add(Box::new(Quad::new(Point3::new(-3.0,-2.0, 5.0), Vec3::new(0.0, 0.0,-4.0), Vec3::new(0.0, 4.0, 0.0), left_red)));
    world.add(Box::new(Quad::new(Point3::new(-2.0,-2.0, 0.0), Vec3::new(4.0, 0.0, 0.0), Vec3::new(0.0, 4.0, 0.0), back_green)));
    world.add(Box::new(Quad::new(Point3::new( 3.0,-2.0, 1.0), Vec3::new(0.0, 0.0, 4.0), Vec3::new(0.0, 4.0, 0.0), right_blue)));
    world.add(Box::new(Quad::new(Point3::new(-2.0, 3.0, 1.0), Vec3::new(4.0, 0.0, 0.0), Vec3::new(0.0, 0.0, 4.0), upper_orange)));
    world.add(Box::new(Quad::new(Point3::new(-2.0,-3.0, 5.0), Vec3::new(4.0, 0.0, 0.0), Vec3::new(0.0, 0.0,-4.0), lower_teal)));
 
    let cam = Camera::new(
        400,
        100,
        50,
        Point3::new(0.0, 0.0, 9.0),
        Point3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
        80.0,
        1.0,
        0.1,
        10.0,
        Color::new(0.70, 0.80, 1.00));

    cam.render(&world);
}

fn simple_light() {
    let mut world = HittableList::new();
 
    let pertext = Rc::new(NoiseTexture::new(4.0));
    world.add(Box::new(Sphere::new(
        Point3::new(0.0, -1000.0, 0.0), 1000.0,
        Rc::new(Lambertian::from_texture(pertext.clone())),
    )));
    world.add(Box::new(Sphere::new(
        Point3::new(0.0, 2.0, 0.0), 2.0,
        Rc::new(Lambertian::from_texture(pertext)),
    )));
 
    let difflight = Rc::new(DiffuseLight::from_color(Color::new(4.0, 4.0, 4.0)));
    world.add(Box::new(Sphere::new(
        Point3::new(0.0, 7.0, 0.0), 2.0,
        difflight.clone(),
    )));
    world.add(Box::new(Quad::new(
        Point3::new(3.0, 1.0, -2.0),
        Vec3::new(2.0, 0.0, 0.0),
        Vec3::new(0.0, 2.0, 0.0),
        difflight,
    )));
 
    let cam = Camera::new(
        400,
        100,
        50,
        Point3::new(26.0, 3.0, 6.0),
        Point3::new(0.0, 2.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
        20.0,
        16.0 / 9.0,
        0.1,
        10.0,
        Color::new(0.0, 0.0, 0.0));

    cam.render(&world);
}

fn cornell_box() {
    let mut world = HittableList::new();
 
    let red   = Rc::new(Lambertian::new(Color::new(0.65, 0.05, 0.05)));
    let white = Rc::new(Lambertian::new(Color::new(0.73, 0.73, 0.73)));
    let green = Rc::new(Lambertian::new(Color::new(0.12, 0.80, 0.15)));
    let light = Rc::new(DiffuseLight::from_color(Color::new(15.0, 15.0, 15.0)));
 
    world.add(Box::new(Quad::new(Point3::new(555.0,   0.0,   0.0), Vec3::new(0.0, 555.0,   0.0), Vec3::new(  0.0, 0.0, 555.0), green)));
    world.add(Box::new(Quad::new(Point3::new(  0.0,   0.0,   0.0), Vec3::new(0.0, 555.0,   0.0), Vec3::new(  0.0, 0.0, 555.0), red)));
    world.add(Box::new(Quad::new(Point3::new(343.0, 554.0, 332.0), Vec3::new(-130.0, 0.0, 0.0), Vec3::new(  0.0, 0.0,-105.0), light)));
    world.add(Box::new(Quad::new(Point3::new(  0.0,   0.0,   0.0), Vec3::new(555.0,   0.0, 0.0), Vec3::new(  0.0, 0.0, 555.0), white.clone())));
    world.add(Box::new(Quad::new(Point3::new(555.0, 555.0, 555.0), Vec3::new(-555.0,  0.0, 0.0), Vec3::new(  0.0, 0.0,-555.0), white.clone())));
    world.add(Box::new(Quad::new(Point3::new(  0.0,   0.0, 555.0), Vec3::new(555.0,   0.0, 0.0), Vec3::new(  0.0, 555.0, 0.0), white.clone())));

    let box1: Box<HittableList> = quad::make_box(Point3::new(0.0,0.0,0.0), Point3::new(165.0,330.0,165.0), white.clone());
    let box1: Box<dyn Hittable> = Box::new(RotateY::new(box1, 15.0));
    let box1: Box<dyn Hittable> = Box::new(Translate::new(box1, Vec3::new(265.0, 0.0, 295.0)));
    world.add(box1);
 
    let box2 = quad::make_box(Point3::new(0.0,0.0,0.0), Point3::new(165.0,165.0,165.0), white.clone());
    let box2: Box<dyn Hittable> = Box::new(RotateY::new(box2, -18.0));
    let box2: Box<dyn Hittable> = Box::new(Translate::new(box2, Vec3::new(130.0, 0.0, 65.0)));
    world.add(box2);

//    world.add(quad::make_box(Point3::new(130.0, 0.0,  65.0), Point3::new(295.0, 165.0, 230.0), white.clone()));
//    world.add(quad::make_box(Point3::new(265.0, 0.0, 295.0), Point3::new(430.0, 330.0, 460.0), white.clone()));

    let cam = Camera::new(
        600,
        200,
        50,
        Point3::new(278.0, 278.0, -800.0),
        Point3::new(278.0, 278.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
        40.0,
        1.0,
        0.1,
        10.0,
        Color::new(0.0, 0.0, 0.0));

    cam.render(&world);
}


fn cornell_smoke() {
    let mut world = HittableList::new();
 
    let red   = Rc::new(Lambertian::new(Color::new(0.65, 0.05, 0.05)));
    let white = Rc::new(Lambertian::new(Color::new(0.73, 0.73, 0.73)));
    let green = Rc::new(Lambertian::new(Color::new(0.12, 0.45, 0.15)));
    let light = Rc::new(DiffuseLight::from_color(Color::new(7.0, 7.0, 7.0)));
 
    world.add(Box::new(Quad::new(Point3::new(555.0,   0.0,   0.0), Vec3::new(  0.0, 555.0,   0.0), Vec3::new(  0.0,   0.0, 555.0), green)));
    world.add(Box::new(Quad::new(Point3::new(  0.0,   0.0,   0.0), Vec3::new(  0.0, 555.0,   0.0), Vec3::new(  0.0,   0.0, 555.0), red)));
    world.add(Box::new(Quad::new(Point3::new(113.0, 554.0, 127.0), Vec3::new(330.0,   0.0,   0.0), Vec3::new(  0.0,   0.0, 305.0), light)));
    world.add(Box::new(Quad::new(Point3::new(  0.0, 555.0,   0.0), Vec3::new(555.0,   0.0,   0.0), Vec3::new(  0.0,   0.0, 555.0), white.clone())));
    world.add(Box::new(Quad::new(Point3::new(  0.0,   0.0,   0.0), Vec3::new(555.0,   0.0,   0.0), Vec3::new(  0.0,   0.0, 555.0), white.clone())));
    world.add(Box::new(Quad::new(Point3::new(  0.0,   0.0, 555.0), Vec3::new(555.0,   0.0,   0.0), Vec3::new(  0.0, 555.0,   0.0), white.clone())));
 
    let box1 = quad::make_box(Point3::new(0.0,0.0,0.0), Point3::new(165.0,330.0,165.0), white.clone());
    let box1: Box<dyn Hittable> = Box::new(RotateY::new(box1, 15.0));
    let box1: Box<dyn Hittable> = Box::new(Translate::new(box1, Vec3::new(265.0, 0.0, 295.0)));
 
    let box2 = quad::make_box(Point3::new(0.0,0.0,0.0), Point3::new(165.0,165.0,165.0), white.clone());
    let box2: Box<dyn Hittable> = Box::new(RotateY::new(box2, -18.0));
    let box2: Box<dyn Hittable> = Box::new(Translate::new(box2, Vec3::new(130.0, 0.0, 65.0)));
 
    world.add(Box::new(ConstantMedium::from_color(box1, 0.01, Color::new(0.0, 0.0, 0.0))));
    world.add(Box::new(ConstantMedium::from_color(box2, 0.01, Color::new(1.0, 1.0, 1.0))));
 
    const IMAGE_WIDTH: usize = 600;
    const SAMPLES_PER_PIXEL: i32 = 200;
    const MAX_DEPTH: i32 = 50;
    let cam = Camera::new(
        IMAGE_WIDTH,
        SAMPLES_PER_PIXEL,
        MAX_DEPTH,
        Point3::new(278.0, 278.0, -800.0),
        Point3::new(278.0, 278.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
        40.0,
        1.0,
        0.1,
        10.0,
        Color::new(0.0, 0.0, 0.0),
    );
 
    cam.render(&world);
}

fn main() {
    match 8 {
        1 => bouncing_spheres(),
        2 => checkered_spheres(),
        3 => earth(),
        4 => perlin_spheres(),
        5 => quads(),
        6 => simple_light(),
        7 => cornell_box(),
        8 => cornell_smoke(),
        _ => {}
    }
}