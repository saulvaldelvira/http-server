use std::sync::{Arc, Mutex};
use job_pool::ThreadPool;

#[test]
fn pool_counter() {
    static N:i16 = 1024;
    let pool = ThreadPool::with_size(32).expect("Expected Ok value");
    let count = Arc::new(Mutex::new(0));

    let inc = |i:i16| {
        for _ in 0..N {
            let count = Arc::clone(&count);
            pool.execute(move || {
                let mut n = count.lock().unwrap();
                *n += i;
            })
        }
    };

    let check = |i:i16| {
        let n = count.lock().unwrap();
        assert_eq!(*n,i);
    };

    inc(1);
    pool.join();
    check(N);

    inc(-1);
    pool.join();
    check(0);
}
