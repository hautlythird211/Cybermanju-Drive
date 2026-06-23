use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use cybermanju_resolutions::writer::ShardWriter;
use std::env;

fn bench_shard_write(c: &mut Criterion) {
    let mut group = c.benchmark_group("shard_write");
    let file_counts = [10, 50, 100];

    for count in file_counts {
        group.bench_with_input(BenchmarkId::new("write", count), &count, |b, &count| {
            b.iter(|| {
                let tmp = env::temp_dir().join(format!("bench_shard_{}.cybermanju", count));
                let mut writer = ShardWriter::new("bench_shard", "root_hash");
                for i in 0..count {
                    let r0 = vec![0u8; 5_000];
                    let r1 = vec![0u8; 20_000];
                    let r2 = vec![0u8; 100_000];
                    let r3 = vec![0u8; 500_000];
                    writer
                        .add_file(
                            &format!("file_{}", i),
                            &format!("photo_{}.jpg", i),
                            "image/jpeg",
                            "/photos",
                            &r0,
                            &r1,
                            &r2,
                            &r3,
                        )
                        .unwrap();
                }
                writer.finalize(&tmp).unwrap();
                let _ = std::fs::remove_file(&tmp);
            });
        });
    }
    group.finish();
}

fn bench_erasure_encode(c: &mut Criterion) {
    let mut group = c.benchmark_group("erasure_encode");
    let data_sizes = [65536, 1_048_576];

    for size in data_sizes {
        let data: Vec<u8> = (0..size).map(|i| (i % 256) as u8).collect();
        let engine = cybermanju_erasure::ShardErasureEngine::new_reed_solomon(4, 2).unwrap();
        group.bench_with_input(BenchmarkId::new("rs_encode", size), &data, |b, data| {
            b.iter(|| engine.encode(data).unwrap());
        });
    }
    group.finish();
}

criterion_group!(benches, bench_shard_write, bench_erasure_encode);
criterion_main!(benches);
