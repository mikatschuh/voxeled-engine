use rand::Rng;

pub fn get_random<T: Ord + rand::distributions::uniform::SampleUniform>(min: T, max: T) -> T {
    rand::thread_rng().gen_range(min..=max)
}

use noise::{NoiseFn, Perlin};

#[derive(Clone, Debug)]
pub struct Noise {
    noise: Perlin,
}

impl Noise {
    pub fn new(seed: u32) -> Self {
        Self {
            noise: Perlin::new(seed),
        }
    }

    pub fn get(&self, x: f64, y: f64, z: f64, space_scale: f64) -> f64 {
        let animated_x = x / space_scale;
        let animated_y = y / space_scale;
        let animated_z = z / space_scale;

        // Variante 1: Zeit direkt als vierte Dimension nutzen
        // let value = self.noise.get([
        //     animated_x, animated_y, animated_z, t, // Zeit als zusätzliche Dimension
        // ]);

        // Oder Variante 2: Bewegte Koordinaten
        let value = self.noise.get([
            animated_x, // Koordinaten bewegen sich mit der Zeit
            animated_y, // verschiedene Faktoren für mehr Variation
            animated_z,
        ]);

        (value + 1.0) * 0.5
    }

    pub fn get_octaves(&self, x: f64, y: f64, z: f64, space_scale: f64, octaves: usize) -> f64 {
        let x = x / space_scale;
        let y = y / space_scale;
        let z = z / space_scale;

        let mut value = 0.0;
        let mut max_value = 0.0;
        let mut amplitude = 1.0;
        let mut frequency = 1.0;
        let persistence = 0.5;

        for _ in 0..octaves {
            value += self.get(x * frequency, y, z * frequency, space_scale) * amplitude;
            max_value += amplitude;
            amplitude *= persistence;
            frequency *= 2.0;
        }

        value / max_value
    }
}
