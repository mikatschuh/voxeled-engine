use glam::IVec3;

use crate::{
    chunk::{ChunkID, Level},
    mesh::Mesh,
    voxel::{self, VoxelData3D, VoxelType},
};

pub type BitMap3D = [[u32; 32]; 32];

fn get_data(level: &Level, chunk_id: ChunkID) -> VoxelData3D {
    level
        .chunk_op(chunk_id, |chunk| *chunk.voxel.read())
        .flatten()
        .unwrap_or_else(|| voxel::fill(VoxelType::Air))
}

pub fn get_axis_aligned_solid_maps(level: &Level, chunk: ChunkID) -> [BitMap3D; 3] {
    let data = get_data(level, chunk);

    let mut x_aligned = [[0; 32]; 32];
    let mut y_aligned = [[0; 32]; 32];
    let mut z_aligned = [[0; 32]; 32];

    // data setup
    for (x, plane) in data.iter().enumerate() {
        for (y, row) in plane.iter().enumerate() {
            for (z, voxel) in row.iter().enumerate() {
                let voxel_is_solid_u32 = voxel.is_solid_u32();

                if voxel_is_solid_u32 > 0 {
                    x_aligned[y][z] |= voxel_is_solid_u32 >> x;
                    y_aligned[z][x] |= voxel_is_solid_u32 >> y;
                    z_aligned[x][y] |= voxel_is_solid_u32 >> z;
                }
            }
        }
    }
    [x_aligned, y_aligned, z_aligned]
}

fn get_x_aligned_solid_map(level: &Level, chunk: ChunkID) -> BitMap3D {
    let data = get_data(level, chunk);

    let mut x_aligned = [[0; 32]; 32];

    // data setup
    for (x, plane) in data.iter().enumerate() {
        for (y, row) in plane.iter().enumerate() {
            for (z, voxel) in row.iter().enumerate() {
                let voxel_is_solid_u32 = voxel.is_solid_u32();

                if voxel_is_solid_u32 > 0 {
                    x_aligned[y][z] |= voxel_is_solid_u32 >> x;
                }
            }
        }
    }
    x_aligned
}

fn get_y_aligned_solid_map(level: &Level, chunk: ChunkID) -> BitMap3D {
    let data = get_data(level, chunk);

    let mut y_aligned = [[0; 32]; 32];

    // data setup
    for (x, plane) in data.iter().enumerate() {
        for (y, row) in plane.iter().enumerate() {
            for (z, voxel) in row.iter().enumerate() {
                let voxel_is_solid_u32 = voxel.is_solid_u32();

                if voxel_is_solid_u32 > 0 {
                    y_aligned[z][x] |= voxel_is_solid_u32 >> y;
                }
            }
        }
    }
    y_aligned
}

fn get_z_aligned_solid_map(level: &Level, chunk: ChunkID) -> BitMap3D {
    let data = get_data(level, chunk);
    let mut z_aligned = [[0; 32]; 32];

    // data setup
    for (x, plane) in data.iter().enumerate() {
        for (y, row) in plane.iter().enumerate() {
            for (z, voxel) in row.iter().enumerate() {
                let voxel_is_solid_u32 = voxel.is_solid_u32();

                if voxel_is_solid_u32 > 0 {
                    z_aligned[x][y] |= voxel_is_solid_u32 >> z;
                }
            }
        }
    }
    z_aligned
}

pub fn map_visible(level: &Level, chunk: ChunkID) -> [BitMap3D; 6] {
    let mut faces = [[[0; 32]; 32]; 6];
    // 0 = -x
    // 1 = +x
    // 2 = -y
    // 3 = +y
    // 4 = -z
    // 5 = +z

    let [x_aligned, y_aligned, z_aligned] = get_axis_aligned_solid_maps(level, chunk);

    let (px, nx, py, ny, pz, nz) = (
        get_x_aligned_solid_map(
            level,
            ChunkID {
                lod: chunk.lod,
                pos: chunk.pos + IVec3::new(1, 0, 0),
            },
        ),
        get_x_aligned_solid_map(
            level,
            ChunkID {
                pos: chunk.pos + IVec3::new(-1, 0, 0),
                lod: chunk.lod,
            },
        ),
        get_y_aligned_solid_map(
            level,
            ChunkID {
                pos: chunk.pos + IVec3::new(0, 1, 0),
                lod: chunk.lod,
            },
        ),
        get_y_aligned_solid_map(
            level,
            ChunkID {
                pos: chunk.pos + IVec3::new(0, -1, 0),
                lod: chunk.lod,
            },
        ),
        get_z_aligned_solid_map(
            level,
            ChunkID {
                pos: chunk.pos + IVec3::new(0, 0, 1),
                lod: chunk.lod,
            },
        ),
        get_z_aligned_solid_map(
            level,
            ChunkID {
                pos: chunk.pos + IVec3::new(0, 0, -1),
                lod: chunk.lod,
            },
        ),
    );
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
    /*
    println!(
        "\
    x:  {}\n\
    -x: {}\n\
    +x: {}\n\n\
    y:  {}\n\
    -y: {}\n\
    +y: {}\n\n\
    z:  {}\n\
    -z: {}\n\
    +z: {}\n\
    ",
        format(x_aligned[0][0]),
        format(faces[0].0[0][0]),
        format(faces[1].0[0][0]),
        format(y_aligned[0][0]),
        format(faces[2].0[0][0]),
        format(faces[3].0[0][0]),
        format(z_aligned[0][0]),
        format(faces[4].0[0][0]),
        format(faces[5].0[0][0])
    );

    fn format(num: u32) -> String {
        let mut out = String::new();
        for i in (0..32).rev() {
            out += match num >> i & 1 {
                0 => "   ",
                1 => "|#|",
                _ => unreachable!(),
            }
        }
        out
    }
    */
    // WARNING! additional step: add non solid blocks back in
    faces
}

const FIRST_BIT: u32 = 0b1000_0000_0000_0000_0000_0000_0000_0000;

pub fn generate_mesh(chunk: ChunkID, data: VoxelData3D, faces: [BitMap3D; 6]) -> Mesh {
    let chunk_pos = chunk.pos << 5;
    let mut mesh = Mesh::with_capacity(100);
    for x in 0..32 {
        for y in 0..32 {
            for z in 0..32 {
                if data[x][y][z] == VoxelType::Air {
                    continue;
                }

                let position: IVec3 =
                    (chunk_pos + IVec3::new(x as i32, y as i32, z as i32)) << chunk.lod;

                if faces[0][y][z] & (FIRST_BIT >> x) != 0 {
                    mesh.add_nx(position, data[x][y][z].texture_id(0), chunk.lod)
                }
                if faces[1][y][z] & (FIRST_BIT >> x) != 0 {
                    mesh.add_px(position, data[x][y][z].texture_id(1), chunk.lod)
                }
                if faces[2][z][x] & (FIRST_BIT >> y) != 0 {
                    mesh.add_ny(position, data[x][y][z].texture_id(2), chunk.lod)
                }
                if faces[3][z][x] & (FIRST_BIT >> y) != 0 {
                    mesh.add_py(position, data[x][y][z].texture_id(3), chunk.lod)
                }
                if faces[4][x][y] & (FIRST_BIT >> z) != 0 {
                    mesh.add_nz(position, data[x][y][z].texture_id(4), chunk.lod)
                }
                if faces[5][x][y] & (FIRST_BIT >> z) != 0 {
                    mesh.add_pz(position, data[x][y][z].texture_id(5), chunk.lod)
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
