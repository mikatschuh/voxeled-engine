use std::thread;

use criterion::{Criterion, criterion_group, criterion_main};
use rtrb::{PopError, PushError};
use voxine::mpsc::{self};

fn run_mpsc_threaded(messages_per_producer: usize, producers: usize, capacity: usize) {
    let (tx, rx) = mpsc::new::<usize>(capacity);

    let mut handles = Vec::with_capacity(producers);
    for producer_id in 0..producers {
        let tx = tx.clone();
        handles.push(thread::spawn(move || {
            for i in 0..messages_per_producer {
                let mut value = i ^ (producer_id << 20);
                loop {
                    match tx.push(value) {
                        Ok(()) => break,
                        Err(PushError::Full(v)) => {
                            value = v;
                            std::hint::spin_loop();
                        }
                    }
                }
            }
        }));
    }
    drop(tx);

    let total = messages_per_producer * producers;
    let mut received = 0usize;
    while received < total {
        match rx.pop() {
            Ok(_) => received += 1,
            Err(PopError::Empty) => std::hint::spin_loop(),
        }
    }

    for handle in handles {
        handle.join().unwrap();
    }
}

fn run_rtrb_threaded(messages: usize, capacity: usize) {
    let (mut tx, mut rx) = rtrb::RingBuffer::new(capacity);

    let producer = thread::spawn(move || {
        for mut value in 0..messages {
            loop {
                match tx.push(value) {
                    Ok(()) => break,
                    Err(rtrb::PushError::Full(v)) => {
                        value = v;
                        std::hint::spin_loop();
                    }
                }
            }
        }
    });

    let mut received = 0usize;
    while received < messages {
        match rx.pop() {
            Ok(_) => received += 1,
            Err(rtrb::PopError::Empty) => std::hint::spin_loop(),
        }
    }

    producer.join().unwrap();
}

fn benchmark_channels(c: &mut Criterion) {
    let mut group = c.benchmark_group("channel_throughput");
    group.sample_size(20);

    for &capacity in &[1024usize, 16_384, 65_536] {
        group.bench_function(
            format!("mpsc_threaded_4p1c_100k_each_cap_{capacity}"),
            |b| {
                b.iter(|| run_mpsc_threaded(100_000, 4, capacity));
            },
        );
    }

    for &capacity in &[1024usize, 16_384, 65_536] {
        group.bench_function(format!("rtrb_threaded_1p1c_200k_cap_{capacity}"), |b| {
            b.iter(|| run_rtrb_threaded(200_000, capacity));
        });
    }

    group.finish();
}

criterion_group!(benches, benchmark_channels);
criterion_main!(benches);
