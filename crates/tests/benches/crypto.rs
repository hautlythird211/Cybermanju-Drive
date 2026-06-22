use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};

fn bench_ml_kem_keygen(c: &mut Criterion) {
    c.bench_function("ml_kem_1024_keygen", |b| {
        b.iter(|| {
            let _keypair = ml_kem::MlKem1024::generate(&mut rand_core::OsRng);
        })
    });
}

fn bench_ml_kem_encap(c: &mut Criterion) {
    let keypair = ml_kem::MlKem1024::generate(&mut rand_core::OsRng);
    c.bench_function("ml_kem_1024_encap", |b| {
        b.iter(|| {
            let _shared = ml_kem::encapsulate(&keypair.public, &mut rand_core::OsRng).unwrap();
        })
    });
}

fn bench_chacha20poly1305(c: &mut Criterion) {
    use chacha20poly1305::{Key, XChaCha20Poly1305, XNonce};
    use chacha20poly1305::aead::Aead;

    let key = Key::from_slice(b"0123456789abcdef0123456789abcdef");
    let cipher = XChaCha20Poly1305::new(key);
    let nonce = XNonce::from_slice(b"0123456789abcdef01234567");

    let data_sizes = [1024, 65536, 1_048_576];
    let mut group = c.benchmark_group("chacha20poly1305");

    for size in data_sizes {
        let data: Vec<u8> = (0..size).map(|i| (i % 256) as u8).collect();
        group.bench_with_input(BenchmarkId::new("encrypt", size), &data, |b, data| {
            b.iter(|| cipher.encrypt(nonce, data.as_slice()).unwrap())
        });
    }
    group.finish();
}

criterion_group!(benches, bench_ml_kem_keygen, bench_ml_kem_encap, bench_chacha20poly1305);
criterion_main!(benches);
