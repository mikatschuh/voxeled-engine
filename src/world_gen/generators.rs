use std::ops::Add;

use super::{Layer, ShapeGenerator};
use crate::{
    ComposeableGenerator, Gen2D, Gen3D, GenBox,
    random::Noise,
    world_gen::{MaterialGenerator, Seed},
};

impl Add for ComposeableGenerator {
    type Output = Self;
    fn add(mut self, mut rhs: Self) -> Self::Output {
        self.gen_stack.append(&mut rhs.gen_stack);

        self
    }
}

impl ComposeableGenerator {
    pub fn gen_3d(gen3d: Gen3D, material: Option<MaterialGenerator>) -> Self {
        Self {
            gen_stack: vec![Layer {
                generator: ShapeGenerator::Gen3D(gen3d),
                material,
            }],
        }
    }

    pub fn gen_2d(gen2d: Gen2D, material: Option<MaterialGenerator>) -> Self {
        Self {
            gen_stack: vec![Layer {
                generator: ShapeGenerator::Gen2D(gen2d),
                material,
            }],
        }
    }

    pub fn gen_box(box_gen: GenBox, material: Option<MaterialGenerator>) -> Self {
        Self {
            gen_stack: vec![Layer {
                generator: ShapeGenerator::Box(box_gen),
                material,
            }],
        }
    }

    pub fn mountains_and_valleys(seed: Seed) -> Self {
        Self {
            gen_stack: vec![Layer {
                generator: ShapeGenerator::Gen2D(Gen2D {
                    noise: Noise::new(seed as u32),
                    x_scale: 20.0,
                    z_scale: 20.0,
                    y_scale: 1200.0,
                    base_height: 0.,
                    octaves: 3,
                }),
                material: Some(MaterialGenerator::new(seed)),
            }],
        }
    }

    pub fn rain_drops(seed: Seed) -> Self {
        Self {
            gen_stack: vec![Layer {
                generator: ShapeGenerator::Gen3D(Gen3D {
                    noise: Noise::new(seed as u32),
                    octaves: 1,
                    x_scale: 5.0,
                    y_scale: 5.0,
                    z_scale: 5.0,
                    exponent: 1.,
                    threshold: 0.8,
                }),
                material: Some(MaterialGenerator::new(seed)),
            }],
        }
    }

    pub fn open_caves(seed: Seed) -> Self {
        Self {
            gen_stack: vec![Layer {
                generator: ShapeGenerator::Gen3D(Gen3D {
                    noise: Noise::new(seed as u32),
                    x_scale: 32.0, // 8.0,
                    y_scale: 32.0,
                    z_scale: 32.0,
                    exponent: 1.,
                    threshold: 0.5,
                    octaves: 9,
                }),
                material: Some(MaterialGenerator::new(seed)),
            }],
        }
    }
}
