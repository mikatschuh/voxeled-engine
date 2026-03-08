use criterion::{Criterion, black_box, criterion_group, criterion_main};
use glam::{IVec3, Vec3};
use voxine::{
    ComposableGenerator, GenBox, Generator, MaterialGenerator, SphereConfig,
    SphereGeneratorAllocations,
};

fn benchmark_chunk_generation(c: &mut Criterion) {
    let seed = 1093029392039201909;
    let generator = ComposableGenerator::gen_box(
        GenBox {
            min: IVec3::new(-100, -100, -100),
            max: IVec3::new(100, 100, 100),
        },
        Some(MaterialGenerator::new(seed)),
    );
    let max_chunks = 1_000_000;
    let mut allocations = SphereGeneratorAllocations::default(max_chunks);

    let mut chunks = vec![];
    SphereConfig {
        full_detail_range: 12.,
        radius: 10_000. / 32.,
        max_chunks: max_chunks,
    }
    .flood_fill(Vec3::ZERO, &mut allocations, |c| chunks.push(c));
    let chunks = chunks.into_iter();
    c.bench_function("ComposableGenerator::generate", |b| {
        b.iter(|| {
            for c in black_box(chunks.clone()) {
                let out = black_box(generator.clone()).generate(c);
                black_box(out);
            }
        });
    });
}

criterion_group!(benches, benchmark_chunk_generation);
criterion_main!(benches);
