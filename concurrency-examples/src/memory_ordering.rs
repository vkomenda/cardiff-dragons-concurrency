use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

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

    println!("{final_x}, {final_y}");
    (final_x, final_y)
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
}
