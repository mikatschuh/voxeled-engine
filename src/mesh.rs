use glam::UVec3;

#[derive(Debug, Clone)]
pub struct MeshUpload {
    pub offsets: [u64; 6],
    pub buf: Box<[u8]>,
}

pub type TextureID = u16;

/// The kind states the orientation and the texture.
/// It has the following layout:
/// |x x x x x y y y y y z z z z z| texture
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
    pub(crate) nx: Vec<Instance>,
    pub(crate) px: Vec<Instance>,

    pub(crate) ny: Vec<Instance>,
    pub(crate) py: Vec<Instance>,

    pub(crate) nz: Vec<Instance>,
    pub(crate) pz: Vec<Instance>,
}

impl Mesh {
    #[allow(unused)]
    pub(crate) fn new() -> Self {
        Self {
            nx: vec![],
            px: vec![],
            ny: vec![],
            py: vec![],
            nz: vec![],
            pz: vec![],
        }
    }

    pub(crate) fn with_capacity(cap: usize) -> Self {
        Self {
            nx: Vec::with_capacity(cap),
            px: Vec::with_capacity(cap),
            ny: Vec::with_capacity(cap),
            py: Vec::with_capacity(cap),
            nz: Vec::with_capacity(cap),
            pz: Vec::with_capacity(cap),
        }
    }

    pub fn bytes(mut self) -> MeshUpload {
        let px_offset = (self.nx.len() << 2) as u64;
        let ny_offset = px_offset + (self.px.len() << 2) as u64;
        let py_offset = ny_offset + (self.ny.len() << 2) as u64;
        let nz_offset = py_offset + (self.py.len() << 2) as u64;
        let pz_offset = nz_offset + (self.nz.len() << 2) as u64;

        let offsets = [0, px_offset, ny_offset, py_offset, nz_offset, pz_offset];

        let mut unified_buffer = Vec::with_capacity(pz_offset as usize + self.pz.len());
        unified_buffer.append(&mut self.nx);
        unified_buffer.append(&mut self.px);
        unified_buffer.append(&mut self.ny);
        unified_buffer.append(&mut self.py);
        unified_buffer.append(&mut self.nz);
        unified_buffer.append(&mut self.pz);

        let slice = unified_buffer.into_boxed_slice();
        MeshUpload {
            offsets,
            buf: bytemuck::cast_slice_box(slice),
        }
    }

    pub(crate) fn add_nx(&mut self, pos: UVec3, texture: TextureID) {
        self.nx.push(Instance {
            kind: compress_data(pos, texture),
        });
    }

    pub(crate) fn add_px(&mut self, pos: UVec3, texture: TextureID) {
        self.px.push(Instance {
            kind: compress_data(pos, texture),
        });
    }

    pub(crate) fn add_ny(&mut self, pos: UVec3, texture: TextureID) {
        self.ny.push(Instance {
            kind: compress_data(pos, texture),
        });
    }

    pub(crate) fn add_py(&mut self, pos: UVec3, texture: TextureID) {
        self.py.push(Instance {
            kind: compress_data(pos, texture),
        });
    }

    pub(crate) fn add_nz(&mut self, pos: UVec3, texture: TextureID) {
        self.nz.push(Instance {
            kind: compress_data(pos, texture),
        });
    }

    pub(crate) fn add_pz(&mut self, pos: UVec3, texture: TextureID) {
        self.pz.push(Instance {
            kind: compress_data(pos, texture),
        });
    }
}

fn compress_data(pos: UVec3, texture: TextureID) -> u32 {
    (pos.x << 27) | (pos.y << 22) | (pos.z << 17) | texture as u32
}
