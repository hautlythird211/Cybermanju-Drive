use criterion::{criterion_group, criterion_main, Criterion};

fn bench_face_cluster_bruteforce(c: &mut Criterion) {
    c.bench_function("face_cluster_bruteforce_100", |b| {
        let entries: Vec<(String, Vec<f32>)> = (0..100)
            .map(|i| {
                let emb: Vec<f32> = (0..512).map(|j| ((i * 512 + j) as f32).sin()).collect();
                (format!("face_{}", i), emb)
            })
            .collect();
        b.iter(|| {
            cybermanju_faces::cluster_bruteforce(&entries, 0.55);
        })
    });
}

fn bench_simhash_build(c: &mut Criterion) {
    c.bench_function("simhash_build_1000", |b| {
        let entries: Vec<(String, Vec<f32>)> = (0..1000)
            .map(|i| {
                let emb: Vec<f32> = (0..512).map(|j| ((i * 512 + j) as f32).sin()).collect();
                (format!("face_{}", i), emb)
            })
            .collect();
        b.iter(|| {
            cybermanju_faces::SimHashIndex::new(&entries);
        })
    });
}

criterion_group!(benches, bench_face_cluster_bruteforce, bench_simhash_build);
criterion_main!(benches);
