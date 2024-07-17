mod actors;
mod loom;
mod memory_ordering;

use dashmap::DashMap;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
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

fn worker_pool_matrix_multiply(a: &[Vec<f64>], b: &[Vec<f64>]) -> Vec<Vec<f64>> {
    let n = a.len();
    let m = b[0].len();
    let p = b.len();

    assert!(
        a[0].len() == p,
        "Matrix dimensions do not match for multiplication!"
    );

    (0..n)
        .into_par_iter()
        .map(|i| {
            (0..m)
                .map(|j| (0..p).map(|k| a[i][k] * b[k][j]).sum())
                .collect()
        })
        .collect()
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
}
