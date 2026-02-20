use crate::{
    chunk::ChunkID,
    random::Noise,
    voxel::{VoxelData3D, VoxelType},
};

type Seed = u64;
pub trait Generator: Clone + Send + Sync + 'static {
    fn generate(&self, chunk_id: ChunkID) -> VoxelData3D;
    fn seed(&self) -> Seed;
}

#[derive(Clone)]
pub struct MountainsAndValleys {
    pub seed: Seed,
    pub noise: Noise,
    pub horizontal_area: f64,
    pub vertical_area: f64,
    pub ground_level: f64,

    pub number_of_octaves: usize,
}

impl MountainsAndValleys {
    pub fn new(seed: Seed) -> Self {
        Self {
            seed,
            noise: Noise::new(seed as u32),
            horizontal_area: 20.0,
            vertical_area: 1200.0,
            ground_level: 0.,
            number_of_octaves: 3,
        }
    }
}

impl Generator for MountainsAndValleys {
    fn generate(&self, chunk_id: ChunkID) -> VoxelData3D {
        let mut voxels = [[[VoxelType::Air; 32]; 32]; 32];

        for (x, plane) in voxels.iter_mut().enumerate() {
            for z in 0..32 {
                let height = self.noise.get_octaves(
                    ((x as i32 + chunk_id.pos.x * 32) << chunk_id.lod) as f64,
                    0.0,
                    ((z as i32 + chunk_id.pos.z * 32) << chunk_id.lod) as f64,
                    self.horizontal_area,
                    self.number_of_octaves,
                );
                assert!(height <= 1.0);
                assert!(height >= 0.0);
                for y in 0..32 {
                    plane[y][z] = if (y as i32 + chunk_id.pos.y * 32) << chunk_id.lod
                        < ((2.0_f64.powf(height as f64) * self.vertical_area) - self.ground_level)
                            as i32
                    {
                        VoxelType::random_weighted()
                    } else {
                        VoxelType::Air
                    }
                }
            }
        }
        voxels
    }
    fn seed(&self) -> Seed {
        self.seed
    }
}

#[derive(Clone)]
pub struct WhiteNoise {
    pub seed: Seed,
}

impl WhiteNoise {
    pub fn new(seed: Seed) -> Self {
        Self { seed }
    }
}

impl Generator for WhiteNoise {
    fn generate(&self, _chunk_id: ChunkID) -> VoxelData3D {
        let mut voxels = [[[VoxelType::Air; 32]; 32]; 32];
        for plane in voxels.iter_mut() {
            for row in plane.iter_mut() {
                for voxel in row.iter_mut() {
                    *voxel = VoxelType::random_weighted()
                }
            }
        }
        voxels
    }
    fn seed(&self) -> Seed {
        self.seed
    }
}

#[derive(Clone)]
pub struct RainDrops {
    pub seed: Seed,
    pub noise: Noise,
    pub horizontal_area: f64,
    pub exponent: i32,
    pub threshold: f64,
    pub number_of_octaves: usize,
}

impl RainDrops {
    pub fn new(seed: Seed) -> Self {
        Self {
            seed,
            noise: Noise::new(seed as u32),
            horizontal_area: 5.0,
            exponent: 1,
            threshold: 0.8,
            number_of_octaves: 1,
        }
    }
}

impl Generator for RainDrops {
    fn generate(&self, chunk_id: ChunkID) -> VoxelData3D {
        let mut voxels = [[[VoxelType::Air; 32]; 32]; 32];
        for (x, plane) in voxels.iter_mut().enumerate() {
            for (y, row) in plane.iter_mut().enumerate() {
                for (z, voxel) in row.iter_mut().enumerate() {
                    let val = self.noise.get_octaves(
                        ((x as i32 + chunk_id.pos.x * 32) << chunk_id.lod) as f64,
                        ((y as i32 + chunk_id.pos.y * 32) << chunk_id.lod) as f64,
                        ((z as i32 + chunk_id.pos.z * 32) << chunk_id.lod) as f64,
                        self.horizontal_area,
                        self.number_of_octaves,
                    );
                    *voxel = if val.powf(self.exponent as f64) > self.threshold {
                        VoxelType::random_weighted()
                    } else {
                        VoxelType::Air
                    }
                }
            }
        }
        voxels
    }
    fn seed(&self) -> Seed {
        self.seed
    }
}

#[derive(Clone)]
struct MaterialGenerator {
    pub noise: Noise,
    pub scale: f64,
    pub threshold: f64,
    pub octaves: usize,
}

impl MaterialGenerator {
    pub fn new(seed: Seed) -> Self {
        Self {
            noise: Noise::new(seed as u32 ^ 0b11010101010101010100011010101010),
            scale: 8.0,
            threshold: 0.6,
            octaves: 3,
        }
    }
}

impl MaterialGenerator {
    fn generate(&self, pos: (f64, f64, f64)) -> VoxelType {
        let mat = self
            .noise
            .get_octaves(pos.0, pos.1, pos.2, self.scale, self.octaves);

        match mat {
            _ if mat >= self.threshold => VoxelType::CrackedStone,
            _ => VoxelType::Stone,
        }
    }
}

#[derive(Clone)]
pub struct OpenCaves {
    pub seed: Seed,
    pub noise: Noise,
    pub horizontal_area: f64,
    pub exponent: i32,
    pub threshold: f64,
    pub number_of_octaves: usize,

    material_generator: MaterialGenerator,
}

impl OpenCaves {
    pub fn new(seed: Seed) -> Self {
        Self {
            seed,
            noise: Noise::new(seed as u32),
            horizontal_area: 32.0, // 8.0,
            exponent: 1,
            threshold: 0.5,
            number_of_octaves: 9,

            material_generator: MaterialGenerator::new(
                seed ^ 0b1101010101010101010001101010101011010101010101010100011010101010,
            ),
        }
    }
}

impl Generator for OpenCaves {
    fn generate(&self, chunk_id: ChunkID) -> VoxelData3D {
        let mut voxels = [[[VoxelType::Air; 32]; 32]; 32];
        for (x, plane) in voxels.iter_mut().enumerate() {
            for (y, row) in plane.iter_mut().enumerate() {
                for (z, voxel) in row.iter_mut().enumerate() {
                    let pos = (
                        ((x as i32 + chunk_id.pos.x * 32) << chunk_id.lod) as f64,
                        ((y as i32 + chunk_id.pos.y * 32) << chunk_id.lod) as f64,
                        ((z as i32 + chunk_id.pos.z * 32) << chunk_id.lod) as f64,
                    );

                    let val = self.noise.get_octaves(
                        pos.0,
                        pos.1,
                        pos.2,
                        self.horizontal_area,
                        self.number_of_octaves,
                    );

                    *voxel = if val.powf(self.exponent as f64) <= self.threshold {
                        VoxelType::Air
                    } else {
                        self.material_generator.generate(pos)
                    }
                }
            }
        }
        voxels
    }
    fn seed(&self) -> Seed {
        self.seed
    }
}

#[derive(Clone)]
pub struct Box {
    pub seed: Seed,
    pub size: f64,

    material_generator: MaterialGenerator,
}

impl Box {
    pub fn new(seed: Seed, size: f64) -> Self {
        Self {
            seed,
            size,
            material_generator: MaterialGenerator::new(seed),
        }
    }
}

impl Generator for Box {
    fn generate(&self, chunk_id: ChunkID) -> VoxelData3D {
        let mut voxels = [[[VoxelType::Air; 32]; 32]; 32];

        for (x, plane) in voxels.iter_mut().enumerate() {
            for (y, row) in plane.iter_mut().enumerate() {
                for (z, voxel) in row.iter_mut().enumerate() {
                    let pos = (
                        ((x as i32 + chunk_id.pos.x * 32) << chunk_id.lod) as f64,
                        ((y as i32 + chunk_id.pos.y * 32) << chunk_id.lod) as f64,
                        ((z as i32 + chunk_id.pos.z * 32) << chunk_id.lod) as f64,
                    );

                    *voxel = if pos.0 < self.size
                        && pos.1 < self.size
                        && pos.2 < self.size
                        && pos.0 >= -self.size
                        && pos.1 >= -self.size
                        && pos.2 >= -self.size
                    {
                        VoxelType::Air
                    } else {
                        self.material_generator.generate(pos)
                    }
                }
            }
        }
        voxels
    }

    fn seed(&self) -> Seed {
        self.seed
    }
}
