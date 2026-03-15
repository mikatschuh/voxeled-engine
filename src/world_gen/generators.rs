use std::ops::Mul;

use glam::IVec3;

use super::{Layer, ShapeGenerator};
use crate::{
    ComposableGenerator, Gen2D, Gen3D, GenBox, random::Noise, voxel::VoxelTypes, world_gen::Seed,
};

impl Mul for ComposableGenerator {
    type Output = Self;
    fn mul(mut self, mut rhs: Self) -> Self::Output {
        self.gen_stack.append(&mut rhs.gen_stack);

        self
    }
}

impl ComposableGenerator {
    pub fn gen_3d(gen3d: Gen3D, material: VoxelTypes) -> Self {
        Self {
            gen_stack: vec![Layer {
                generator: ShapeGenerator::Gen3D(gen3d),
                material,
            }],
        }
    }

    pub fn gen_2d(gen2d: Gen2D, material: VoxelTypes) -> Self {
        Self {
            gen_stack: vec![Layer {
                generator: ShapeGenerator::Gen2D(gen2d),
                material,
            }],
        }
    }

    pub fn gen_box(min: IVec3, max: IVec3, material: VoxelTypes) -> Self {
        Self {
            gen_stack: vec![Layer {
                generator: ShapeGenerator::Box(GenBox {
                    invert: true,
                    min,
                    max,
                }),
                material,
            }],
        }
    }

    pub fn gen_cube(min: IVec3, max: IVec3, material: VoxelTypes) -> Self {
        Self {
            gen_stack: vec![Layer {
                generator: ShapeGenerator::Box(GenBox {
                    invert: false,
                    min,
                    max,
                }),
                material,
            }],
        }
    }

    pub fn full(material: VoxelTypes) -> Self {
        Self {
            gen_stack: vec![Layer {
                generator: ShapeGenerator::Full,
                material: material,
            }],
        }
    }

    pub fn dirt(seed: Seed) -> Self {
        Self::full(VoxelTypes::Dirt0)
            * Self::gen_3d(
                Gen3D {
                    noise: Noise::new(seed as u32),
                    octaves: 2,
                    x_scale: 30.,
                    y_scale: 30.,
                    z_scale: 30.,
                    exponent: 1.,
                    threshold: 0.5,
                },
                VoxelTypes::Dirt1,
            )
    }

    pub fn mountains_and_valleys(seed: Seed) -> Self {
        Self::dirt(seed)
            * Self::gen_2d(
                Gen2D {
                    invert: true,

                    noise: Noise::new(seed as u32),
                    x_scale: 20.0,
                    z_scale: 20.0,
                    y_scale: 1200.0,
                    base_height: 0.,
                    octaves: 3,
                },
                VoxelTypes::Air,
            )
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
                material: VoxelTypes::Stone,
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
                material: VoxelTypes::Stone,
            }],
        }
    }
}
