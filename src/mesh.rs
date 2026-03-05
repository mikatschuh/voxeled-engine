use glam::{IVec3, UVec3};
use std::ops;

use crate::engine::LodLevel;

pub type TextureID = u16;

/// The kind states the orientation and the texture.
/// It has the following layout:
/// ORIENT| LOD |x x x x x y y y y y z z z z z| texture
/// |0|0|0|0|0|0|0|0|0|0|0|0|0|0|0|0|0|0|0|0|0|0|0|0|0|0|0|0|0|0|0|0|
///
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Instance {
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
            attributes: &[wgpu::VertexAttribute {
                offset: mem::size_of::<IVec3>() as wgpu::BufferAddress,
                shader_location: 3,
                format: wgpu::VertexFormat::Uint32,
            }],
        }
    }
}

#[derive(Debug, Clone)]
pub struct Mesh {
    pub buf: Vec<Instance>,
}

impl ops::AddAssign<Self> for Mesh {
    fn add_assign(&mut self, mut other: Self) {
        self.buf.append(&mut other.buf);
    }
}

impl ops::Add for Mesh {
    type Output = Self;

    fn add(mut self, mut other: Self) -> Self {
        self.buf.append(&mut other.buf);
        self
    }
}

impl Mesh {
    pub fn new() -> Self {
        Self { buf: vec![] }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            buf: Vec::with_capacity(capacity),
        }
    }

    pub fn len(&self) -> usize {
        self.buf.len()
    }

    pub fn add_nx(&mut self, pos: UVec3, texture: TextureID, lod: LodLevel) {
        self.buf.push(Instance {
            kind: ((lod as u32) << 26)
                | (pos.x << 21)
                | (pos.y << 16)
                | (pos.z << 11)
                | texture as u32,
        });
    }

    pub fn add_px(&mut self, pos: UVec3, texture: TextureID, lod: LodLevel) {
        self.buf.push(Instance {
            kind: (0b001 << 29)
                | ((lod as u32) << 26)
                | (pos.x << 21)
                | (pos.y << 16)
                | (pos.z << 11)
                | texture as u32,
        });
    }

    pub fn add_ny(&mut self, pos: UVec3, texture: TextureID, lod: LodLevel) {
        self.buf.push(Instance {
            kind: (0b010 << 29)
                | ((lod as u32) << 26)
                | (pos.x << 21)
                | (pos.y << 16)
                | (pos.z << 11)
                | texture as u32,
        });
    }

    pub fn add_py(&mut self, pos: UVec3, texture: TextureID, lod: LodLevel) {
        self.buf.push(Instance {
            kind: (0b011 << 29)
                | ((lod as u32) << 26)
                | (pos.x << 21)
                | (pos.y << 16)
                | (pos.z << 11)
                | texture as u32,
        });
    }

    pub fn add_nz(&mut self, pos: UVec3, texture: TextureID, lod: LodLevel) {
        self.buf.push(Instance {
            kind: (0b100 << 29)
                | ((lod as u32) << 26)
                | (pos.x << 21)
                | (pos.y << 16)
                | (pos.z << 11)
                | texture as u32,
        });
    }

    pub fn add_pz(&mut self, pos: UVec3, texture: TextureID, lod: LodLevel) {
        self.buf.push(Instance {
            kind: (0b101 << 29)
                | ((lod as u32) << 26)
                | (pos.x << 21)
                | (pos.y << 16)
                | (pos.z << 11)
                | texture as u32,
        });
    }
}
