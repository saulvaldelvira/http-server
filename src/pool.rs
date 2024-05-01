mod worker;

use std::sync::{mpsc,Arc,Mutex};
use worker::{Job, Worker};
use crate::{ServerError,Result};

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
}

impl ThreadPool {
    /// Create a new ThreadPool.
    ///
    /// The size is the number of threads in the pool.
    ///
    /// # Returns
    /// A [Result]<[ThreadPool],[ServerError]>
    pub fn new(size: usize) -> Result<ThreadPool> {
        if size == 0 {
            return Err(ServerError::from_str("Invalid size: 0"));
        }
        let (sender,receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        let mut workers = Vec::with_capacity(size);
        for id in 0..size {
            let worker = Worker::new(id, Arc::clone(&receiver));
            workers.push(worker);
        }
        Ok(ThreadPool {workers,sender:Some(sender)})
    }
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static
    {
        let job = Box::new(f);
        self.sender
            .as_ref()
            .unwrap()
            .send(job)
            .unwrap();
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
