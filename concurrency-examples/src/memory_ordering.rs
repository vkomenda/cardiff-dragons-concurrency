use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use std::sync::Arc;
use std::thread;

static X: AtomicBool = AtomicBool::new(false);

fn relaxed_ordering() -> (bool, bool) {
    // Shared flags
    let x = Arc::new(AtomicBool::new(false));
    let y = Arc::new(AtomicBool::new(false));

    let t1 = {
        let x = Arc::clone(&x);
        let y = Arc::clone(&y);
        thread::spawn(move || {
            let a = y.load(Ordering::Relaxed);
            x.store(a, Ordering::Relaxed);
        })
    };

    let t2 = {
        let x = Arc::clone(&x);
        let y = Arc::clone(&y);
        thread::spawn(move || {
            let _b = x.load(Ordering::Relaxed);
            y.store(true, Ordering::Relaxed);
        })
    };

    t1.join().unwrap();
    t2.join().unwrap();

    let final_x = x.load(Ordering::Relaxed);
    let final_y = y.load(Ordering::Relaxed);

    (final_x, final_y)
}

fn acqrel_relaxed_ordering() -> i32 {
    let x = Arc::new(AtomicBool::new(false));
    let y = Arc::new(AtomicBool::new(false));
    let z = Arc::new(AtomicI32::new(0));

    let t1 = {
        let x = Arc::clone(&x);
        thread::spawn(move || {
            x.store(true, Ordering::Release);
        })
    };

    let t2 = {
        let y = Arc::clone(&y);
        thread::spawn(move || {
            y.store(true, Ordering::Release);
        })
    };

    let t3 = {
        let x = Arc::clone(&x);
        let y = Arc::clone(&y);
        let z = Arc::clone(&z);
        thread::spawn(move || {
            while !x.load(Ordering::Acquire) {}
            if y.load(Ordering::Acquire) {
                z.fetch_add(1, Ordering::Relaxed);
            }
        })
    };

    let t4 = {
        let x = Arc::clone(&x);
        let y = Arc::clone(&y);
        let z = Arc::clone(&z);
        thread::spawn(move || {
            while !y.load(Ordering::Acquire) {}
            if x.load(Ordering::Acquire) {
                z.fetch_add(1, Ordering::Relaxed);
            }
        })
    };

    t1.join().unwrap();
    t2.join().unwrap();
    t3.join().unwrap();
    t4.join().unwrap();

    z.load(Ordering::Acquire)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relaxed_ordering_reorders() {
        let results: Vec<_> = (0..100).map(|_| relaxed_ordering()).collect();
        let first_r = results[0];
        for &r in &results[1..] {
            if r != first_r {
                return;
            }
        }
        panic!("All results are the same");
    }

    #[test]
    fn acqrel_relaxed_ordering_reorders() {
        let results: Vec<_> = (0..100).map(|_| acqrel_relaxed_ordering()).collect();
        assert!(results.contains(&2));
        assert!(results.contains(&1));
        // assert!(results.contains(&0));
    }
}
