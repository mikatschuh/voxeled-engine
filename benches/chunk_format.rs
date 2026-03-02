use criterion::{Criterion, black_box, criterion_group, criterion_main};
use glam::UVec3;
use rand::{random, thread_rng};
use rand_distr::{Distribution, Exp};
use voxine::{Chunk, VoxelType};

fn benchmark_chunk_editing(c: &mut Criterion) {
    let mut group = c.benchmark_group("channel_throughput");
    group.sample_size(20);

    let mut rng = thread_rng();
    let exp = Exp::new(0.7).unwrap();
    let replaces = (0..100_000)
        .map(|_| {
            (
                UVec3::new(
                    random::<u32>() & 31,
                    random::<u32>() & 31,
                    random::<u32>() & 31,
                ),
                (exp.sample(&mut rng) as f32).clamp(0., u16::MAX as f32) as u16,
            )
        })
        .collect::<Vec<(UVec3, VoxelType)>>();

    let mut naiv_chunk = [0_u16; 32_768];

    #[inline(always)]
    fn coords_to_1d_index(coord: UVec3) -> usize {
        (coord.x * 32 * 32 + coord.y * 32 + coord.z) as usize
    }

    group.bench_function(format!("naiv set {}", replaces.len()), |b| {
        b.iter(|| {
            for replace in black_box(replaces.iter()) {
                naiv_chunk[coords_to_1d_index(replace.0)] = replace.1
            }
        });
    });

    let mut optimized_chunk = Chunk::from_buffer(&[[[0_u16; 32]; 32]; 32]);

    group.bench_function(format!("Chunk::set {}", replaces.len()), |b| {
        b.iter(|| {
            for replace in black_box(replaces.iter()) {
                optimized_chunk.set(replace.0, replace.1);
            }
        });
    });
}

criterion_group!(benches, benchmark_chunk_editing);
criterion_main!(benches);
