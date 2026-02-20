use std::{
    collections::HashMap,
    sync::{
        Arc,
        atomic::{AtomicU8, Ordering},
    },
};

use glam::{IVec3, Vec3};
use parking_lot::{RwLock, RwLockReadGuard};

use crate::{frustum::LodLevel, mesh::Mesh, meshing::BitMap3D, voxel::VoxelData3D};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct ChunkID {
    pub lod: LodLevel,
    pub pos: IVec3,
}

impl ChunkID {
    pub fn new(lod: LodLevel, pos: IVec3) -> Self {
        Self { lod, pos }
    }

    pub fn total_pos(self) -> IVec3 {
        self.pos << self.lod
    }

    pub fn parent(self) -> Self {
        Self {
            lod: self.lod + 1,
            pos: self.pos >> 1,
        }
    }

    pub fn size(self) -> f32 {
        (1 << self.lod) as f32
    }

    pub fn from_pos(v: Vec3, lod: LodLevel) -> Self {
        Self {
            lod,
            pos: v.floor().as_ivec3(),
        }
    }
}

impl From<Vec3> for ChunkID {
    fn from(value: Vec3) -> Self {
        Self {
            lod: 0,
            pos: value.floor().as_ivec3(),
        }
    }
}

pub struct Level {
    chunks: RwLock<HashMap<ChunkID, Chunk>>,
}

impl Level {
    pub fn new() -> Self {
        Self {
            chunks: RwLock::new(HashMap::new()),
        }
    }

    pub fn with_capacity(cap: usize) -> Self {
        Self {
            chunks: RwLock::new(HashMap::with_capacity(cap)),
        }
    }

    pub fn contains(&self, chunk_id: ChunkID) -> bool {
        self.chunks.read().contains_key(&chunk_id)
    }

    pub fn insert(&self, chunk_id: ChunkID, chunk: Chunk) -> Result<(), ()> {
        let mut lock = self.chunks.write();
        if lock.contains_key(&chunk_id) {
            return Err(());
        }
        lock.insert(chunk_id, chunk);
        Ok(())
    }

    #[allow(unused)]
    fn lock<'a>(&'a self) -> RwLockReadGuard<'a, HashMap<ChunkID, Chunk>> {
        self.chunks.read()
    }

    #[inline(always)]
    pub fn chunk_op<A>(&self, chunk_id: ChunkID, f: impl FnOnce(&Chunk) -> A) -> Option<A> {
        let lock = self.chunks.read();
        let chunk = lock.get(&chunk_id)?;
        Some(f(chunk))
    }
}

pub struct Chunk {
    pub voxel_state: AtomicDataState,
    pub voxel: RwLock<Option<VoxelData3D>>,

    pub occl_state: AtomicDataState,
    pub occl: RwLock<Option<[BitMap3D; 6]>>, // [neg. x, pos. x, neg. y, pos. y, neg. z, pos. z]

    pub mesh_state: AtomicDataState,
    pub mesh: Arc<RwLock<Mesh>>,
}

impl Chunk {
    pub fn new(voxel_state: DataState) -> Self {
        Self {
            voxel_state: AtomicDataState::new(voxel_state),
            voxel: RwLock::new(None),

            occl_state: AtomicDataState::new(DataState::Done),
            occl: RwLock::new(None),

            mesh_state: AtomicDataState::new(DataState::Done),
            mesh: Arc::new(RwLock::new(Mesh::new())),
        }
    }

    pub fn write_voxel(&self, voxel: VoxelData3D) {
        *self.voxel.write() = Some(voxel);
        self.occl_state.finish_generating();

        self.occl_state.mark_dirty();
        self.mesh_state.mark_dirty();
    }

    pub fn write_occl(&self, occl: [BitMap3D; 6]) {
        *self.occl.write() = Some(occl);
        self.occl_state.finish_generating();

        self.mesh_state.mark_dirty();
    }

    pub fn write_mesh(&self, mesh: Mesh) {
        *self.mesh.write() = mesh;
        self.mesh_state.finish_generating();
    }
}

pub enum DataState {
    Done,
    Dirty,
    Generating,
    GeneratingDirty,
}

pub struct AtomicDataState {
    data: AtomicU8,
}

impl AtomicDataState {
    pub fn new(state: DataState) -> Self {
        Self {
            data: AtomicU8::new(state as u8), // everything fully up-to-date
        }
    }

    pub fn load(&self) -> u8 {
        self.data.load(Ordering::Acquire)
    }

    pub fn is_done(&self) -> bool {
        self.data.load(Ordering::Acquire) == DataState::Done as u8
    }

    pub fn try_start_generating(&self) -> Result<(), ()> {
        loop {
            match self.data.load(Ordering::Acquire) {
                state if state == DataState::Dirty as u8 => {
                    if self
                        .data
                        .compare_exchange(
                            DataState::Dirty as u8,
                            DataState::Generating as u8,
                            Ordering::AcqRel,
                            Ordering::Acquire,
                        )
                        .is_ok()
                    {
                        return Ok(());
                    }
                }
                _ => return Err(()),
            }
        }
    }

    pub fn finish_generating(&self) {
        loop {
            match self.data.load(Ordering::Acquire) {
                state if state == DataState::Generating as u8 => {
                    if self
                        .data
                        .compare_exchange(
                            DataState::Generating as u8,
                            DataState::Done as u8,
                            Ordering::AcqRel,
                            Ordering::Acquire,
                        )
                        .is_ok()
                    {
                        return;
                    }
                }
                state if state == DataState::GeneratingDirty as u8 => {
                    if self
                        .data
                        .compare_exchange(
                            DataState::GeneratingDirty as u8,
                            DataState::Dirty as u8,
                            Ordering::AcqRel,
                            Ordering::Acquire,
                        )
                        .is_ok()
                    {
                        return;
                    }
                }
                _ => return,
            }
        }
    }

    pub fn mark_dirty(&self) {
        loop {
            match self.data.load(Ordering::Acquire) {
                state if state == DataState::Done as u8 => {
                    if self
                        .data
                        .compare_exchange(
                            DataState::Done as u8,
                            DataState::Dirty as u8,
                            Ordering::AcqRel,
                            Ordering::Acquire,
                        )
                        .is_ok()
                    {
                        return;
                    }
                }
                state if state == DataState::Generating as u8 => {
                    if self
                        .data
                        .compare_exchange(
                            DataState::Generating as u8,
                            DataState::GeneratingDirty as u8,
                            Ordering::AcqRel,
                            Ordering::Acquire,
                        )
                        .is_ok()
                    {
                        return;
                    }
                }
                _ => return,
            }
        }
    }
}
