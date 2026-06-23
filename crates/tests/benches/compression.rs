use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use cybermanju_compression::TripleCompressor;

fn bench_triple_compress(c: &mut Criterion) {
    let compressor = TripleCompressor::new();
    let sizes = [1024, 65536, 1_048_576];

    let mut group = c.benchmark_group("triple_compress");
    for size in sizes {
        let data: Vec<u8> = (0..size).map(|i| (i % 256) as u8).collect();
        group.bench_with_input(BenchmarkId::new("compress", size), &data, |b, data| {
            b.iter(|| compressor.compress_triple(data).unwrap())
        });
    }
    group.finish();
}

fn bench_triple_decompress(c: &mut Criterion) {
    let compressor = TripleCompressor::new();
    let sizes = [1024, 65536, 1_048_576];

    let mut group = c.benchmark_group("triple_decompress");
    for size in sizes {
        let data: Vec<u8> = (0..size).map(|i| (i % 256) as u8).collect();
        let (compressed, _) = compressor.compress_triple(&data).unwrap();
        group.bench_with_input(
            BenchmarkId::new("decompress", size),
            &compressed,
            |b, compressed| b.iter(|| compressor.decompress_triple(compressed).unwrap()),
        );
    }
    group.finish();
}

fn bench_blake3_hash(c: &mut Criterion) {
    let mut group = c.benchmark_group("blake3_hash");
    let sizes = [1024, 65536, 1_048_576, 10_485_760];

    for size in sizes {
        let data: Vec<u8> = (0..size).map(|i| (i % 256) as u8).collect();
        group.bench_with_input(BenchmarkId::new("hash", size), &data, |b, data| {
            b.iter(|| blake3::hash(data))
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_triple_compress,
    bench_triple_decompress,
    bench_blake3_hash
);
criterion_main!(benches);
