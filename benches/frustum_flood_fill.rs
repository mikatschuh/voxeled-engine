use std::f32::consts::FRAC_PI_3;

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use glam::Vec3;
use voxine::Frustum;

fn benchmark_flood_fill(c: &mut Criterion) {
    c.bench_function("frustum_flood_fill", |b| {
        let frustum = Frustum {
            cam_pos: Vec3::ZERO,
            direction: Vec3::new(0.3, 0.7, 0.6).normalize(),
            fov: FRAC_PI_3,
            aspect_ratio: 16. / 9.,
            max_chunks: 5000,
            max_distance: 48.,
            full_detail_range: 12.,
        };

        b.iter(|| {
            let out = black_box(frustum).flood_fill();
            black_box(out);
        });
    });
}

criterion_group!(benches, benchmark_flood_fill);
criterion_main!(benches);
