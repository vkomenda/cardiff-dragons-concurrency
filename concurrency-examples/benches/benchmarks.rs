use concurrency_examples::{
    matrix_multiply, matrix_multiply_avx, matrix_multiply_avx_rayon, matrix_multiply_rayon,
};
use criterion::{criterion_group, criterion_main, Criterion};

fn generate_matrices(size: usize) -> (Vec<f32>, Vec<f32>) {
    let mut a = vec![0.0; size * size];
    let mut b = vec![0.0; size * size];

    for i in 0..size {
        for j in 0..size {
            a[i * size + j] = (i + j) as f32;
            b[i * size + j] = (i * j) as f32;
        }
    }

    (a, b)
}

fn bench_simple(c: &mut Criterion) {
    let size = 256;
    let (a, b) = generate_matrices(size);

    c.bench_function("matrix_multiply_simple", |bencher| {
        bencher.iter(|| matrix_multiply(&a, &b, size, size, size))
    });
}

fn bench_rayon(c: &mut Criterion) {
    let size = 256;
    let (a, b) = generate_matrices(size);

    c.bench_function("matrix_multiply_rayon", |bencher| {
        bencher.iter(|| matrix_multiply_rayon(&a, &b, size, size, size))
    });
}

fn bench_avx(c: &mut Criterion) {
    let size = 256;
    let (a, b) = generate_matrices(size);

    c.bench_function("matrix_multiply_avx", |bencher| {
        bencher.iter(|| matrix_multiply_avx(&a, &b, size, size, size))
    });
}

fn bench_avx_rayon(c: &mut Criterion) {
    let size = 256;
    let (a, b) = generate_matrices(size);

    c.bench_function("matrix_multiply_avx_rayon", |bencher| {
        bencher.iter(|| matrix_multiply_avx_rayon(&a, &b, size, size, size))
    });
}

criterion_group!(
    benches,
    bench_simple,
    bench_rayon,
    bench_avx,
    bench_avx_rayon
);
criterion_main!(benches);
