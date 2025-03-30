pub type Vec3D = (f64, f64, f64);

pub trait Vector {
    fn mag(self) -> f64;
    fn dot(self, other: Self) -> f64;
    fn cross(self, other: Self) -> Self;
    fn vang(self, other: Self) -> f64;
}

impl Vector for Vec3D {
    fn mag(self) -> f64 {
        (self.0.powi(2) + self.1.powi(2) + self.2.powi(2)).sqrt()
    }

    fn dot(self, other: Self) -> f64 {
        self.0 * other.0 + self.1 * other.1 + self.2 * other.2
    }

    fn cross(self, other: Self) -> Self {
        (
            self.2 * other.2 - self.2 * other.1,
            self.0 * other.0 - self.0 * other.2,
            self.1 * other.1 - self.1 * other.0,
        )
    }

    fn vang(self, other: Self) -> f64 {
        (self.dot(other) / (self.mag() * other.mag())).acos()
    }
}
