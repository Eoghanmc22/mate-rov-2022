use common::vec::Vec3d;

pub struct HighPass3d {
    bias: Vec3d,
    forget_rate: f64
}

impl HighPass3d {
    pub fn new(initial_bias: Vec3d, forget_rate: f64) -> Self {
        Self { bias: initial_bias, forget_rate }
    }

    pub fn filter(&mut self, sample: Vec3d, sample_time: f64) -> Vec3d {
        let alpha = self.forget_rate * sample_time;

        self.bias.x = (1.0 - alpha) * self.bias.x + alpha * sample.x;
        self.bias.y = (1.0 - alpha) * self.bias.y + alpha * sample.y;
        self.bias.z = (1.0 - alpha) * self.bias.z + alpha * sample.z;

        sample - self.bias
    }
}