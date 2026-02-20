use glam::IVec3;
use std::ops;

use crate::frustum::LodLevel;

pub type TextureID = u16;

/// The kind states the orientation and the texture.
/// It has the following layout:
/// ```
///                             LODs|                        texture|
/// |0|0|0|0|0|0|0|0|0|0|0|0|0|0|0|0|0|0|0|0|0|0|0|0|0|0|0|0|0|0|0|0|
/// ```
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Instance {
    pub pos: IVec3,
    pub kind: u32,
}
unsafe impl bytemuck::Pod for Instance {}
unsafe impl bytemuck::Zeroable for Instance {}

impl Instance {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            // We need to switch from using a step mode of Vertex to Instance
            // This means that our shaders will only change to use the next
            // instance when the shader starts processing a new instance
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                // A mat4 takes up 4 vertex slots as it is technically 4 vec4s. We need to define a slot
                // for each vec4. We'll have to reassemble the mat4 in the shader.
                wgpu::VertexAttribute {
                    offset: 0,
                    // While our vertex shader only uses locations 0, and 1 now, in later tutorials, we'll
                    // be using 2, 3, and 4, for Vertex. We'll start at slot 5, not conflict with them later
                    shader_location: 2,
                    format: wgpu::VertexFormat::Sint32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<IVec3>() as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Uint32,
                },
            ],
        }
    }
}

#[derive(Debug, Clone)]
pub struct Mesh {
    pub nx: Vec<Instance>,
    pub px: Vec<Instance>,
    pub ny: Vec<Instance>,
    pub py: Vec<Instance>,
    pub nz: Vec<Instance>,
    pub pz: Vec<Instance>,
}

impl ops::AddAssign<Self> for Mesh {
    fn add_assign(&mut self, mut other: Self) {
        self.nx.append(&mut other.nx);
        self.px.append(&mut other.px);
        self.ny.append(&mut other.ny);
        self.py.append(&mut other.py);
        self.nz.append(&mut other.nz);
        self.pz.append(&mut other.pz);
    }
}

impl ops::Add for Mesh {
    type Output = Self;

    fn add(mut self, mut other: Self) -> Self {
        self.nx.append(&mut other.nx);
        self.px.append(&mut other.px);
        self.ny.append(&mut other.ny);
        self.py.append(&mut other.py);
        self.nz.append(&mut other.nz);
        self.pz.append(&mut other.pz);
        self
    }
}

impl Mesh {
    pub fn new() -> Self {
        Self {
            nx: vec![],
            px: vec![],
            ny: vec![],
            py: vec![],
            nz: vec![],
            pz: vec![],
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            nx: Vec::with_capacity(capacity),
            px: Vec::with_capacity(capacity),
            ny: Vec::with_capacity(capacity),
            py: Vec::with_capacity(capacity),
            nz: Vec::with_capacity(capacity),
            pz: Vec::with_capacity(capacity),
        }
    }

    pub fn len(&self) -> usize {
        self.nx.len()
            + self.nx.len()
            + self.px.len()
            + self.ny.len()
            + self.py.len()
            + self.nz.len()
            + self.pz.len()
    }

    pub fn add_nx(&mut self, pos: IVec3, texture: TextureID, lod: LodLevel) {
        self.nx.push(Instance {
            pos,
            kind: ((lod as u32) << 16) | texture as u32,
        });
    }

    pub fn add_px(&mut self, pos: IVec3, texture: TextureID, lod: LodLevel) {
        self.px.push(Instance {
            pos,
            kind: ((lod as u32) << 16) | texture as u32,
        });
    }

    pub fn add_ny(&mut self, pos: IVec3, texture: TextureID, lod: LodLevel) {
        self.ny.push(Instance {
            pos,
            kind: ((lod as u32) << 16) | texture as u32,
        });
    }

    pub fn add_py(&mut self, pos: IVec3, texture: TextureID, lod: LodLevel) {
        self.py.push(Instance {
            pos,
            kind: ((lod as u32) << 16) | texture as u32,
        });
    }

    pub fn add_nz(&mut self, pos: IVec3, texture: TextureID, lod: LodLevel) {
        self.nz.push(Instance {
            pos,
            kind: ((lod as u32) << 16) | texture as u32,
        });
    }

    pub fn add_pz(&mut self, pos: IVec3, texture: TextureID, lod: LodLevel) {
        self.pz.push(Instance {
            pos,
            kind: ((lod as u32) << 16) | texture as u32,
        });
    }
}
