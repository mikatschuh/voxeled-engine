use std::f32::consts::FRAC_PI_3;

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use glam::{IVec3, Vec3};
use voxine::{
    ComposableGenerator, Frustum, FrustumAllocations, GenBox, Generator, MaterialGenerator,
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
    let max_chunks = 5000;
    let mut allocations = FrustumAllocations::default(max_chunks);

    let frustum = Frustum {
        cam_pos: Vec3::ZERO,
        direction: Vec3::new(0.3, 0.7, 0.6).normalize(),
        fov: FRAC_PI_3,
        aspect_ratio: 16. / 9.,
        max_chunks,
        max_distance: 48.,
        full_detail_range: 12.,
    };
    frustum.flood_fill(&mut allocations);
    let mut chunks = allocations.chunks.into_iter().cycle();
    c.bench_function("ComposableGenerator::generate", |b| {
        b.iter(|| {
            let out = black_box(generator.clone()).generate(chunks.next().unwrap());
            black_box(out);
        });
    });
}

criterion_group!(benches, benchmark_chunk_generation);
criterion_main!(benches);
