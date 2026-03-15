use glam::UVec3;

use crate::{
    chunk::{DenseChunk, coords_to_1d_index, idx_to_coord},
    mesh::Mesh,
    voxel::{self, VoxelTypes},
};

pub type BitMap2D = [u32; 32];
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

pub fn get_edges([x_aligned, y_aligned, z_aligned]: [BitMap3D; 3]) -> [BitMap2D; 6] {
    [
        z_aligned[0],
        z_aligned[31],
        x_aligned[0],
        x_aligned[31],
        y_aligned[0],
        y_aligned[31],
    ]
}

/// 0 = -x
/// 1 = +x
/// 2 = -y
/// 3 = +y
/// 4 = -z
/// 5 = +z
pub fn map_visible(
    [x_aligned, y_aligned, z_aligned]: &[BitMap3D; 3],
    [nx, px, ny, py, nz, pz]: &[BitMap2D; 6],
) -> [BitMap3D; 6] {
    let mut faces: [BitMap3D; 6] = [[[0; 32]; 32]; 6];
    for i in 0..32 {
        for j in 0..32 {
            faces[0][i][j] =
                x_aligned[i][j] & !((x_aligned[i][j] >> 1) | ((bit_index(nx[i], j) as u32) << 31));
            faces[1][i][j] =
                x_aligned[i][j] & !((x_aligned[i][j] << 1) | bit_index(px[i], j) as u32);
            faces[2][i][j] =
                y_aligned[i][j] & !((y_aligned[i][j] >> 1) | ((bit_index(ny[i], j) as u32) << 31));
            faces[3][i][j] =
                y_aligned[i][j] & !((y_aligned[i][j] << 1) | (bit_index(py[i], j) as u32));
            faces[4][i][j] =
                z_aligned[i][j] & !((z_aligned[i][j] >> 1) | ((bit_index(nz[i], j) as u32) << 31));
            faces[5][i][j] =
                z_aligned[i][j] & !((z_aligned[i][j] << 1) | (bit_index(pz[i], j) as u32));
        }
    }
    faces
}

const FIRST_BIT: u32 = 0b1000_0000_0000_0000_0000_0000_0000_0000;
#[inline]
fn bit_index(x: u32, i: usize) -> bool {
    x & (FIRST_BIT >> i) != 0
}

pub fn generate_mesh(data: &DenseChunk, faces: [BitMap3D; 6]) -> Mesh {
    let mut mesh = Mesh::with_capacity(100);
    for x in 0..32_usize {
        for y in 0..32_usize {
            for z in 0..32_usize {
                let pos = UVec3::new(x as u32, y as u32, z as u32);
                let voxel = data[coords_to_1d_index(pos)];

                if voxel == VoxelTypes::Air as u16 {
                    continue;
                }

                if bit_index(faces[0][y][z], x) {
                    mesh.add_nx(pos, voxel::texture_id(voxel, 0))
                }
                if bit_index(faces[1][y][z], x) {
                    mesh.add_px(pos, voxel::texture_id(voxel, 1))
                }
                if bit_index(faces[2][z][x], y) {
                    mesh.add_ny(pos, voxel::texture_id(voxel, 2))
                }
                if bit_index(faces[3][z][x], y) {
                    mesh.add_py(pos, voxel::texture_id(voxel, 3))
                }
                if bit_index(faces[4][x][y], z) {
                    mesh.add_nz(pos, voxel::texture_id(voxel, 4))
                }
                if bit_index(faces[5][x][y], z) {
                    mesh.add_pz(pos, voxel::texture_id(voxel, 5))
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
