mod worker;
use std::sync::{mpsc, Arc, Condvar, Mutex};
use worker::{Job, Worker};
use crate::{ServerError,Result};

type Semaphore = Arc<(Mutex<u16>,Condvar)>;

/// Thread Pool
///
/// A thread pool coordinates a group of threads to run
/// taks in parallel.
///
/// # Example
/// ```
/// use http_server::pool::ThreadPool;
///
/// let pool = ThreadPool::new(32).expect("Error creating pool");
/// pool.execute(|| println!("Hello world!"));
/// ```
pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
    semaphore: Semaphore,
}

impl ThreadPool {
    /// Create a new ThreadPool.
    ///
    /// The size is the number of threads in the pool.
    ///
    /// # Returns
    /// A [Result]<[ThreadPool],[ServerError]>
    ///
    pub fn new(size: usize) -> Result<ThreadPool> {
        if size == 0 {
            return Err(ServerError::from_str("Invalid size: 0"));
        }
        let (sender,receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        let semaphore = Arc::new((Mutex::new(0),Condvar::new()));
        let mut workers = Vec::with_capacity(size);
        for id in 0..size {
            let worker = Worker::new(id,receiver.clone(),semaphore.clone());
            workers.push(worker);
        }
        Ok(ThreadPool {workers,sender:Some(sender),semaphore})
    }
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static
    {
        let mut counter = self.semaphore.0.lock().unwrap();
        *counter += 1;
        let job = Box::new(f);
        self.sender
            .as_ref()
            .unwrap()
            .send(job)
            .unwrap();
    }
    /// Waits for all the jobs in the pool to finish
    pub fn join(&self) {
        let (lock,condv) = &*self.semaphore;
        let counter = lock.lock().unwrap();
        let _guard = condv.wait_while(counter, |n| *n > 0).unwrap();
    }
}

impl Drop for ThreadPool  {
    fn drop(&mut self) {
        drop(self.sender.take());
        self.workers
            .iter_mut()
            .for_each(Worker::shutdown);
    }
}

#[cfg(test)]
mod test {
    use std::sync::{Arc, Mutex};
    use super::ThreadPool;

    #[test]
    fn pool_size_0() {
        match ThreadPool::new(0) {
            Ok(_) => panic!("Expected Err value"),
            Err(err) => assert_eq!(err.get_message(),"Invalid size: 0"),
        };
    }

    #[test]
    fn pool_counter() {
        static N:i16 = 1024;
        let pool = ThreadPool::new(32).expect("Expected Ok value");
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
}
