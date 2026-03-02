use criterion::{Criterion, black_box, criterion_group, criterion_main};
use glam::UVec3;
use rand::{Rng, random, thread_rng};
use voxine::{Chunk, VoxelType};

fn exp_u16<R: Rng + ?Sized>(rng: &mut R, lambda: f64) -> u16 {
    let max = u16::MAX as f64;
    let u: f64 = rng.r#gen(); // [0,1)
    // Trunkierte Exp auf [0,1], dann auf [0, u16::MAX] skalieren
    let x01 = -((1.0 - u * (1.0 - (-lambda).exp())).ln()) / lambda;
    (x01 * max).round() as u16
}

fn benchmark_chunk_editing(c: &mut Criterion) {
    let mut group = c.benchmark_group("channel_throughput");
    group.sample_size(20);

    let mut rng = thread_rng();
    let replaces = (0..100_000)
        .map(|_| {
            (
                UVec3::new(
                    random::<u32>() & 31,
                    random::<u32>() & 31,
                    random::<u32>() & 31,
                ),
                (exp_u16(&mut rng, 32.0) as f32).clamp(0., u16::MAX as f32) as u16,
            )
        })
        .collect::<Vec<(UVec3, VoxelType)>>();

    let mut naiv_chunk = [0_u16; 32 * 32 * 32];

    #[inline(always)]
    fn coords_to_1d_index(coord: UVec3) -> usize {
        (coord.x * 32 * 32 + coord.y * 32 + coord.z) as usize
    }

    group.bench_function(format!("naiv set {} replaces", replaces.len()), |b| {
        b.iter(|| {
            for replace in black_box(replaces.iter()) {
                naiv_chunk[coords_to_1d_index(replace.0)] = replace.1
            }
        });
    });

    let mut optimized_chunk = Chunk::from_buffer(&[0_u16; 32 * 32 * 32]);

    group.bench_function(format!("Chunk::set {} replaces", replaces.len()), |b| {
        b.iter(|| {
            for replace in black_box(replaces.iter()) {
                optimized_chunk.set(replace.0, replace.1);
            }
        });
    });

    assert_eq!(optimized_chunk.to_buffer(), naiv_chunk);
    println!(
        "\nnaiv memory usage: {}kB\noptimized memory usage: {}kB\n",
        (32 * 32 * 32 * 2) as f32 / 1000.,
        optimized_chunk.calculate_memory_usage() as f32 / 1000.
    );
}

criterion_group!(benches, benchmark_chunk_editing);
criterion_main!(benches);
