mod worker;

use std::sync::{mpsc,Arc,Mutex};
pub use super::error::ServerError as ThreadPoolError;
pub type Result<T> = std::result::Result<T,ThreadPoolError>;
use worker::{Job, Worker};

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
    /// A Result<ThreadPool,ThreadPoolError>
    ///
    pub fn new(size: usize) -> Result<ThreadPool> {
        if size == 0 {
            return Err(ThreadPoolError::from_str("Invalid size: 0"));
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
