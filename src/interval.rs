
#[derive(Copy, Clone, Default)]
pub struct Interval {
    pub min: f64,
    pub max: f64,
}

impl Interval {
    pub const fn new(min: f64, max: f64) -> Interval {
        Self { min, max }
    }

    pub fn size(&self) -> f64 {
        self.max - self.min 
    }

    pub fn contains(&self, x: f64) -> bool {
        self.min <= x && x <= self.max
    }

    pub fn surrounds(&self, x: f64) -> bool {
        return self.min < x && x < self.max;
    }

    pub fn clamp(&self, x: f64)  -> f64 {
        if x < self.min {
            return self.min;
        }
        if x > self.max {
            return self.max;
        }
        x
    }

    pub fn expand(&self, delta: f64) -> Interval {
        let padding = delta/2.0;
        return Interval::new(self.min - padding, self.max + padding);
    }

    pub fn from_intervals(a: Interval, b: Interval) -> Self {
        // Create the interval tightly enclosing the two input intervals.
        Self {
            min: a.min.min(b.min),
            max: a.max.max(b.max),
        }
    }
}

impl std::ops::Add<f64> for Interval {
    type Output = Interval;
    fn add(self, displacement: f64) -> Interval {
        Interval::new(self.min + displacement, self.max + displacement)
    }
}
 
impl std::ops::Add<Interval> for f64 {
    type Output = Interval;
    fn add(self, ival: Interval) -> Interval {
        ival + self
    }
}

pub const EMPTY : Interval = Interval::new(f64::INFINITY, f64::NEG_INFINITY);

//const interval interval::universe = interval(-infinity, +infinity);