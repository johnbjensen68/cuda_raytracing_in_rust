use rand::{RngExt};
 
// Constants
 
pub use std::f64::consts::PI;
pub use std::f64::INFINITY;
 
// Utility functions
 
pub fn degrees_to_radians(degrees: f64) -> f64 {
    degrees * PI / 180.0
}
 
pub fn random_double() -> f64 {
    // Return a random real in [0.0, 1.0)
    rand::rng().random()
}
 
pub fn random_int(min: i32, max: i32) -> i32 {
    // Returns a random integer in [min,max].
    random_double_range(min as f64, (max + 1) as f64) as i32
}

pub fn random_double_range(min: f64, max: f64) -> f64 {
    // Return a random real in [min, max)
    min + (max - min) * random_double()
}

pub fn clamp(x: f64, min: f64, max: f64) -> f64 {
    if x < min {
        return min;
    }
    if x > max {
        return max;
    }
    x
}