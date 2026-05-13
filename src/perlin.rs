use crate::common;
use crate::vec3;

const POINT_COUNT: usize = 256;
 
pub struct Perlin {
    randvec: Vec<vec3::Vec3>,
    perm_x: Vec<usize>,
    perm_y: Vec<usize>,
    perm_z: Vec<usize>,
}
 
impl Perlin {
    pub fn new() -> Self {
        let randvec: Vec<vec3::Vec3> = (0..POINT_COUNT)
            .map(|_| vec3::unit_vector(vec3::Vec3::random_range(-1.0, 1.0)))
            .collect();
        Self {
            randvec,
            perm_x: Self::generate_perm(),
            perm_y: Self::generate_perm(),
            perm_z: Self::generate_perm(),
        }
    }

 
    fn generate_perm() -> Vec<usize> {
        let mut p: Vec<usize> = (0..POINT_COUNT).collect();
        for i in (1..POINT_COUNT).rev() {
            let target =common:: random_int(0, i as i32) as usize;
            p.swap(i, target);
        }
        p
    }

    pub fn turbulence(&self, p: &vec3::Point3, depth: usize) -> f64 {
        let mut accum = 0.0;
        let mut temp_p = *p;
        let mut weight = 1.0;
        for _ in 0..depth {
            accum += weight * self.noise(&temp_p);
            weight *= 0.5;
            temp_p = 2.0 * temp_p;
        }
        accum.abs()
    }
    
    pub fn noise(&self, p: &vec3::Point3) -> f64 {
        let u = p.x() - p.x().floor();
        let v = p.y() - p.y().floor();
        let w = p.z() - p.z().floor();
        let i = p.x().floor() as i32;
        let j = p.y().floor() as i32;
        let k = p.z().floor() as i32;
        let mut c = [[[vec3::Vec3::zero(); 2]; 2]; 2];
        for di in 0..2i32 {
            for dj in 0..2i32 {
                for dk in 0..2i32 {
                    c[di as usize][dj as usize][dk as usize] = self.randvec[
                        self.perm_x[((i + di) & 255) as usize]
                            ^ self.perm_y[((j + dj) & 255) as usize]
                            ^ self.perm_z[((k + dk) & 255) as usize]
                    ];
                }
            }
        }
        self.perlin_interp(c, u, v, w)
    }
 
    fn perlin_interp(&self, c: [[[vec3::Vec3; 2]; 2]; 2], u: f64, v: f64, w: f64) -> f64 {
        let uu = u * u * (3.0 - 2.0 * u);
        let vv = v * v * (3.0 - 2.0 * v);
        let ww = w * w * (3.0 - 2.0 * w);
        let mut accum = 0.0;
        for i in 0..2 {
            for j in 0..2 {
                for k in 0..2 {
                    let weight = vec3::Vec3::new(u - i as f64, v - j as f64, w - k as f64);
                    accum += (i as f64 * uu + (1 - i) as f64 * (1.0 - uu))
                        * (j as f64 * vv + (1 - j) as f64 * (1.0 - vv))
                        * (k as f64 * ww + (1 - k) as f64 * (1.0 - ww))
                        * vec3::dot(c[i][j][k], weight);
                }
            }
        }
        accum
    }
}