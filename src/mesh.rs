use glam::UVec3;

pub type TextureID = u16;

/// The kind states the orientation and the texture.
/// It has the following layout:
/// ORIENT|x x x x x y y y y y z z z z z| texture
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
                offset: 0,
                shader_location: 1,
                format: wgpu::VertexFormat::Uint32,
            }],
        }
    }
}

#[derive(Debug, Clone)]
pub struct Mesh {
    pub(crate) buf: Vec<Instance>,
}

impl Mesh {
    #[allow(unused)]
    pub(crate) fn new() -> Self {
        Self { buf: vec![] }
    }

    pub(crate) fn with_capacity(capacity: usize) -> Self {
        Self {
            buf: Vec::with_capacity(capacity),
        }
    }

    #[allow(unused)]
    pub fn len_in_bytes(&self) -> usize {
        self.buf.len() << 2 // multiply by four
    }

    pub fn bytes(&self) -> &[u8] {
        bytemuck::cast_slice(&self.buf)
    }

    pub(crate) fn add_nx(&mut self, pos: UVec3, texture: TextureID) {
        self.buf.push(Instance {
            kind: (pos.x << 24) | (pos.y << 19) | (pos.z << 14) | texture as u32,
        });
    }

    pub(crate) fn add_px(&mut self, pos: UVec3, texture: TextureID) {
        self.buf.push(Instance {
            kind: (0b001 << 29) | (pos.x << 24) | (pos.y << 19) | (pos.z << 14) | texture as u32,
        });
    }

    pub(crate) fn add_ny(&mut self, pos: UVec3, texture: TextureID) {
        self.buf.push(Instance {
            kind: (0b010 << 29) | (pos.x << 24) | (pos.y << 19) | (pos.z << 14) | texture as u32,
        });
    }

    pub(crate) fn add_py(&mut self, pos: UVec3, texture: TextureID) {
        self.buf.push(Instance {
            kind: (0b011 << 29) | (pos.x << 24) | (pos.y << 19) | (pos.z << 14) | texture as u32,
        });
    }

    pub(crate) fn add_nz(&mut self, pos: UVec3, texture: TextureID) {
        self.buf.push(Instance {
            kind: (0b100 << 29) | (pos.x << 24) | (pos.y << 19) | (pos.z << 14) | texture as u32,
        });
    }

    pub(crate) fn add_pz(&mut self, pos: UVec3, texture: TextureID) {
        self.buf.push(Instance {
            kind: (0b101 << 29) | (pos.x << 24) | (pos.y << 19) | (pos.z << 14) | texture as u32,
        });
    }
}
