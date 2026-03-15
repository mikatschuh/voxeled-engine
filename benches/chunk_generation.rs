use criterion::{Criterion, black_box, criterion_group, criterion_main};
use glam::Vec3;
use voxine::{ComposableGenerator, Generator, SphereGeneratorAllocations};

fn benchmark_chunk_generation(c: &mut Criterion) {
    let seed = 1039030930193019;
    let generator = ComposableGenerator::mountains_and_valleys(seed);
    let max_chunks = 1_000_000;
    let mut allocations = SphereGeneratorAllocations::default(max_chunks);

    let mut chunks = vec![];

    allocations.flood_fill(Vec3::ZERO, 5., 10_000. / 32., max_chunks, |c| {
        chunks.push(c)
    });
    let mut chunks = chunks.into_iter().cycle();
    c.bench_function("ComposableGenerator::generate", |b| {
        b.iter(|| {
            let out = black_box(generator.clone()).generate(black_box(chunks.next().unwrap()));
            black_box(out);
        });
    });
}

criterion_group!(benches, benchmark_chunk_generation);
criterion_main!(benches);
