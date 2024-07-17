#![feature(portable_simd)]

mod actors;
mod loom;
mod memory_ordering;

use dashmap::DashMap;
use rayon::prelude::*;
use std::simd::{f32x8, Simd};
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread;

fn shared_mem_mutex() -> usize {
    let count = Arc::new(Mutex::new(0));

    let mut handles = vec![];

    for _ in 0..10 {
        let count = Arc::clone(&count);
        let handle = thread::spawn(move || {
            let mut num = count.lock().unwrap();
            *num += 1;
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let result: MutexGuard<usize> = count.lock().unwrap();
    *result // Deref implementation gets the lock's data
}

fn shared_mem_dashmap() -> usize {
    let count = Arc::new(DashMap::new());
    count.insert("value", 0);

    let mut handles = vec![];

    for _ in 0..10 {
        let count = Arc::clone(&count);
        let handle = thread::spawn(move || {
            let mut value = count.get_mut("value").unwrap();
            *value += 1;
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let result = count.get("value").unwrap();
    *result
}

/// Multiplies two matrices.
///
/// # Arguments
///
/// * `a` - Left-hand-side matrix.
/// * `b` - Right-hand-side matrix.
/// * `m` - Number of rows in `a`.
/// * `n` - Number of columns in `a` / Number of rows in `b`.
/// * `p` - Number of columns in `b`.
///
/// # Returns
///
/// The resultant matrix after multiplication.
pub fn matrix_multiply(a: &[f32], b: &[f32], m: usize, n: usize, p: usize) -> Vec<f32> {
    let mut result = vec![0.0; m * p];

    // Iterate over the rows of matrix `a`
    for i in 0..m {
        // Iterate over the columns of matrix `b`
        for j in 0..p {
            let mut sum = 0.0;

            // Perform the dot product of the row of `a` and column of `b`
            for k in 0..n {
                sum += a[i * n + k] * b[k * p + j];
            }

            // Store the computed value in the result matrix
            result[i * p + j] = sum;
        }
    }

    result
}

pub fn matrix_multiply_rayon(a: &[f32], b: &[f32], m: usize, n: usize, p: usize) -> Vec<f32> {
    let mut result = vec![0.0; m * p];

    result
        .par_chunks_mut(p)
        .enumerate()
        .for_each(|(i, result_row)| {
            for j in 0..p {
                let mut sum = 0.0;
                for k in 0..n {
                    sum += a[i * n + k] * b[k * p + j];
                }
                result_row[j] = sum;
            }
        });

    result
}

/// Multiplies two matrices using AVX instructions.
pub fn matrix_multiply_avx(a: &[f32], b: &[f32], m: usize, n: usize, p: usize) -> Vec<f32> {
    let mut result = vec![0.0; m * p];

    // Iterate over the rows of matrix `a`
    for i in 0..m {
        // Process each row in `a`
        for k in 0..n {
            let a_elem = a[i * n + k];
            let a_vec = f32x8::splat(a_elem);

            // Iterate over the columns of matrix `b`
            for j in (0..p).step_by(8) {
                // Prepare a chunk of 8 elements from `b`
                let mut padded_chunk = [0.0; 8];
                let remaining = (p - j).min(8);
                padded_chunk[..remaining].copy_from_slice(&b[k * p + j..k * p + j + remaining]);

                let b_chunk = Simd::from_array(padded_chunk);

                // Load the current values in the result matrix
                let mut result_chunk = Simd::from_array([0.0; 8]);
                let mut result_tmp = [0.0; 8];
                result_tmp[..remaining].copy_from_slice(&result[i * p + j..i * p + j + remaining]);
                result_chunk = Simd::from_array(result_tmp);

                // Multiply and accumulate
                result_chunk += a_vec * b_chunk;

                // Store the result back
                result[i * p + j..i * p + j + remaining]
                    .copy_from_slice(&result_chunk.to_array()[..remaining]);
            }
        }
    }

    result
}

/// Multiplies two matrices using AVX instructions. Uses a worker pool to parallelise the outer
/// loop.
pub fn matrix_multiply_avx_rayon(a: &[f32], b: &[f32], m: usize, n: usize, p: usize) -> Vec<f32> {
    let mut result = vec![0.0; m * p];

    // Parallel iteration over rows of matrix `a`
    result
        .par_chunks_mut(p)
        .enumerate()
        .for_each(|(i, result_row)| {
            for k in 0..n {
                let a_elem = a[i * n + k];
                let a_vec = f32x8::splat(a_elem);

                // Iterate over the columns of matrix `b`
                for j in (0..p).step_by(8) {
                    // Prepare a chunk of 8 elements from `b`
                    let mut padded_chunk = [0.0; 8];
                    let remaining = (p - j).min(8);
                    padded_chunk[..remaining].copy_from_slice(&b[k * p + j..k * p + j + remaining]);

                    let b_chunk = Simd::from_array(padded_chunk);

                    // Load the current values in the result matrix
                    let mut result_chunk = Simd::from_array([0.0; 8]);
                    let mut result_tmp = [0.0; 8];
                    result_tmp[..remaining].copy_from_slice(&result_row[j..j + remaining]);
                    result_chunk = Simd::from_array(result_tmp);

                    // Multiply and accumulate
                    result_chunk += a_vec * b_chunk;

                    // Store the result back
                    result_row[j..j + remaining]
                        .copy_from_slice(&result_chunk.to_array()[..remaining]);
                }
            }
        });

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shared_mem_mutex_correct() {
        assert_eq!(shared_mem_mutex(), 10);
    }

    #[test]
    fn shared_mem_dashmap_correct() {
        assert_eq!(shared_mem_dashmap(), 10);
    }

    #[test]
    fn worker_pool_matrix_multiply_correct() {
        let a = vec![vec![1.0, 2.0], vec![3.0, 4.0]];
        let b = vec![vec![5.0, 6.0], vec![7.0, 8.0]];

        let result = worker_pool_matrix_multiply(&a, &b);
        assert_eq!(result, vec![vec![19.0, 22.0], vec![43.0, 50.0]]);
    }

    #[test]
    fn matrix_multiply_correct() {
        let a = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
        let b = vec![7.0, 8.0, 9.0, 10.0, 11.0, 12.0];

        let m = 2; // Number of rows in `a`
        let n = 3; // Number of columns in `a` / Number of rows in `b`
        let p = 2; // Number of columns in `b`

        let expected_result = vec![
            58.0, 64.0, // Row 1 of the result matrix
            139.0, 154.0, // Row 2 of the result matrix
        ];

        let result = matrix_multiply(&a, &b, m, n, p);
        assert_eq!(result, expected_result);

        let result_avx = matrix_multiply_avx(&a, &b, m, n, p);
        assert_eq!(result_avx, expected_result);

        let result_avx_rayon = matrix_multiply_avx(&a, &b, m, n, p);
        assert_eq!(result_avx_rayon, expected_result);
    }
}
