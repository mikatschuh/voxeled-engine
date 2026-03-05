use glam::{IVec3, UVec3};

use crate::{
    ChunkID,
    chunk::{DenseChunk, coords_to_1d_index, idx_to_coord},
    mesh::Mesh,
    voxel::{self, VoxelTypes},
};

pub type BitMap3D = [[u32; 32]; 32];

pub fn get_axis_aligned_solid_maps(data: &DenseChunk) -> [BitMap3D; 3] {
    let mut x_aligned = [[0; 32]; 32];
    let mut y_aligned = [[0; 32]; 32];
    let mut z_aligned = [[0; 32]; 32];

    // data setup
    for (i, voxel) in data.iter().enumerate() {
        let UVec3 { x, y, z } = idx_to_coord(i);

        let voxel_is_solid_u32 = voxel::is_solid_u32(*voxel);

        if voxel_is_solid_u32 > 0 {
            x_aligned[y as usize][z as usize] |= voxel_is_solid_u32 >> x;
            y_aligned[z as usize][x as usize] |= voxel_is_solid_u32 >> y;
            z_aligned[x as usize][y as usize] |= voxel_is_solid_u32 >> z;
        }
    }
    [x_aligned, y_aligned, z_aligned]
}

/// 0 = -x
/// 1 = +x
/// 2 = -y
/// 3 = +y
/// 4 = -z
/// 5 = +z
pub fn map_visible(
    [x_aligned, y_aligned, z_aligned]: &[BitMap3D; 3],
    [nx, px, ny, py, nz, pz]: &[BitMap3D; 6],
) -> [BitMap3D; 6] {
    let mut faces = [[[0; 32]; 32]; 6];
    for i in 0..32 {
        for j in 0..32 {
            faces[0][i][j] = x_aligned[i][j] & !((x_aligned[i][j] >> 1) | (nx[i][j] << 31));
            faces[1][i][j] = x_aligned[i][j] & !((x_aligned[i][j] << 1) | (px[i][j] >> 31));
            faces[2][i][j] = y_aligned[i][j] & !((y_aligned[i][j] >> 1) | (ny[i][j] << 31));
            faces[3][i][j] = y_aligned[i][j] & !((y_aligned[i][j] << 1) | (py[i][j] >> 31));
            faces[4][i][j] = z_aligned[i][j] & !((z_aligned[i][j] >> 1) | (nz[i][j] << 31));
            faces[5][i][j] = z_aligned[i][j] & !((z_aligned[i][j] << 1) | (pz[i][j] >> 31));
        }
    }
    faces
}

const FIRST_BIT: u32 = 0b1000_0000_0000_0000_0000_0000_0000_0000;

pub fn generate_mesh(chunk: ChunkID, data: &DenseChunk, faces: [BitMap3D; 6]) -> Mesh {
    let mut mesh = Mesh::with_capacity(100);
    for x in 0..32_usize {
        for y in 0..32_usize {
            for z in 0..32_usize {
                let pos = UVec3::new(x as u32, y as u32, z as u32);
                let voxel = data[coords_to_1d_index(pos)];

                if voxel == VoxelTypes::Air as u16 {
                    continue;
                }

                if faces[0][y][z] & (FIRST_BIT >> x) != 0 {
                    mesh.add_nx(pos, voxel::texture_id(voxel, 0), chunk.lod)
                }
                if faces[1][y][z] & (FIRST_BIT >> x) != 0 {
                    mesh.add_px(pos, voxel::texture_id(voxel, 1), chunk.lod)
                }
                if faces[2][z][x] & (FIRST_BIT >> y) != 0 {
                    mesh.add_ny(pos, voxel::texture_id(voxel, 2), chunk.lod)
                }
                if faces[3][z][x] & (FIRST_BIT >> y) != 0 {
                    mesh.add_py(pos, voxel::texture_id(voxel, 3), chunk.lod)
                }
                if faces[4][x][y] & (FIRST_BIT >> z) != 0 {
                    mesh.add_nz(pos, voxel::texture_id(voxel, 4), chunk.lod)
                }
                if faces[5][x][y] & (FIRST_BIT >> z) != 0 {
                    mesh.add_pz(pos, voxel::texture_id(voxel, 5), chunk.lod)
                }
            }
        }
    }
    mesh
}

/* Cullign Algorithm

integer:

goal: find voxels that arent covered on the left.

#.#..###.###.##.
|              \

>>

.#.#..###.###.##
|              \

!

#.#.##...#...#..
|              \

& with #.#..###.###.##.

#.#..#...#...#..
================
*/
