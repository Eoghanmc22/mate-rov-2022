use glam::*;

pub struct HighPass3d {
    bias: Option<Vec3>,
    forget_rate: f32
}

impl HighPass3d {
    pub fn new(initial_bias: Vec3, forget_rate: f32) -> Self {
        Self { bias: Some(initial_bias), forget_rate }
    }

    pub fn auto_bias(forget_rate: f32) -> Self {
        Self { bias: None, forget_rate }
    }

    pub fn filter(&mut self, sample: Vec3, sample_time: f32) -> Vec3 {
        if self.bias.is_none() {
            self.bias = Some(sample.clone());
        }

        let alpha = self.forget_rate * sample_time;
        let bias = self.bias.as_mut().unwrap();

        bias.x = (1.0 - alpha) * bias.x + alpha * sample.x;
        bias.y = (1.0 - alpha) * bias.y + alpha * sample.y;
        bias.z = (1.0 - alpha) * bias.z + alpha * sample.z;

        println!("bias: {:?}", bias);

        sample - *bias
    }
}