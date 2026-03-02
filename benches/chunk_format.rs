use criterion::{BenchmarkId, Criterion, Throughput, black_box, criterion_group, criterion_main};
use glam::UVec3;
use rand::{Rng, SeedableRng, rngs::StdRng};
use voxine::{Chunk, VoxelType};

const CHUNK_SIZE: usize = 32;
const CHUNK_VOLUME: usize = CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE;
const OPS: usize = 100_000;

#[inline(always)]
fn coords_to_1d_index(coord: UVec3) -> usize {
    (coord.x * CHUNK_SIZE as u32 * CHUNK_SIZE as u32 + coord.y * CHUNK_SIZE as u32 + coord.z)
        as usize
}

fn gen_ops(seed: u64, ops: usize, value_range: u16) -> Vec<(UVec3, VoxelType)> {
    let mut rng = StdRng::seed_from_u64(seed);
    (0..ops)
        .map(|_| {
            (
                UVec3::new(
                    rng.gen_range(0..CHUNK_SIZE as u32),
                    rng.gen_range(0..CHUNK_SIZE as u32),
                    rng.gen_range(0..CHUNK_SIZE as u32),
                ),
                rng.gen_range(0..value_range),
            )
        })
        .collect()
}

fn bench_bulk_random_writes(c: &mut Criterion) {
    let mut group = c.benchmark_group("chunk_set_bulk_random");
    group.sample_size(30);
    group.throughput(Throughput::Elements(OPS as u64));

    let packed_ops = gen_ops(0xA11CE, OPS, 16);
    let dense_ops = gen_ops(0xBEE5, OPS, u16::MAX);

    let mut naive = [0_u16; CHUNK_VOLUME];
    group.bench_with_input(
        BenchmarkId::new("naive_dense_array", OPS),
        &packed_ops,
        |b, ops| {
            b.iter(|| {
                for &(coord, voxel) in black_box(ops.iter()) {
                    naive[coords_to_1d_index(coord)] = voxel;
                }
            });
        },
    );

    let mut packed_chunk = Chunk::from_buffer(&[0_u16; CHUNK_VOLUME]);
    group.bench_with_input(
        BenchmarkId::new("chunk_set_packed", OPS),
        &packed_ops,
        |b, ops| {
            b.iter(|| {
                for &(coord, voxel) in black_box(ops.iter()) {
                    packed_chunk.set(coord, voxel);
                }
            });
        },
    );

    let mut dense_init = [0_u16; CHUNK_VOLUME];
    for (i, v) in dense_init.iter_mut().enumerate() {
        *v = (i as u16).wrapping_mul(13).wrapping_add(7);
    }
    let mut dense_fallback_chunk = Chunk::from_buffer(&dense_init);
    group.bench_with_input(
        BenchmarkId::new("chunk_set_dense_fallback", OPS),
        &dense_ops,
        |b, ops| {
            b.iter(|| {
                for &(coord, voxel) in black_box(ops.iter()) {
                    dense_fallback_chunk.set(coord, voxel);
                }
            });
        },
    );

    group.finish();
}

fn bench_single_write_hot_path(c: &mut Criterion) {
    let mut group = c.benchmark_group("chunk_set_single_write");
    group.sample_size(40);
    group.throughput(Throughput::Elements(1));

    let packed_ops = gen_ops(0x5EED, OPS, 16);
    let dense_ops = gen_ops(0xD3E5E, OPS, u16::MAX);
    let mut idx = 0usize;

    let mut naive = [0_u16; CHUNK_VOLUME];
    group.bench_function("naive_dense_array", |b| {
        b.iter(|| {
            let (coord, voxel) = black_box(packed_ops[idx % packed_ops.len()]);
            idx = idx.wrapping_add(1);
            naive[coords_to_1d_index(coord)] = voxel;
        });
    });

    let mut chunk = Chunk::from_buffer(&[0_u16; CHUNK_VOLUME]);
    let mut idx2 = 0usize;
    group.bench_function("chunk_set_packed", |b| {
        b.iter(|| {
            let (coord, voxel) = black_box(packed_ops[idx2 % packed_ops.len()]);
            idx2 = idx2.wrapping_add(1);
            chunk.set(coord, voxel);
        });
    });

    let mut dense_init = [0_u16; CHUNK_VOLUME];
    for (i, v) in dense_init.iter_mut().enumerate() {
        *v = (i as u16).wrapping_mul(13).wrapping_add(7);
    }
    let mut chunk_dense_fallback = Chunk::from_buffer(&dense_init);
    let mut idx3 = 0usize;
    group.bench_function("chunk_set_dense_fallback", |b| {
        b.iter(|| {
            let (coord, voxel) = black_box(dense_ops[idx3 % dense_ops.len()]);
            idx3 = idx3.wrapping_add(1);
            chunk_dense_fallback.set(coord, voxel);
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_bulk_random_writes,
    bench_single_write_hot_path
);
criterion_main!(benches);
