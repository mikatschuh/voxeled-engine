use std::f32::consts::FRAC_PI_3;

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use glam::Vec3;
use rand::random;
use voxine::Frustum;

fn benchmark_flood_fill(c: &mut Criterion) {
    c.bench_function("frustum_flood_fill", |b| {
        b.iter(|| {
            black_box(Frustum {
                cam_pos: Vec3::ZERO,
                direction: Vec3::new(random(), random(), random()).normalize(),
                fov: FRAC_PI_3,
                aspect_ratio: 16. / 9.,
                max_chunks: 1_000_000,
                max_distance: 48.,
                full_detail_range: 12.,
            })
            .flood_fill()
        })
    });
}

criterion_group!(benches, benchmark_flood_fill);
criterion_main!(benches);
