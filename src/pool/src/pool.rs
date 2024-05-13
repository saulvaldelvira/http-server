use std::sync::{mpsc, Arc, Condvar, Mutex};
use crate::worker::{Job, Worker};
use crate::{PoolConfig, Result, Semaphore};

/// Thread Pool
///
/// A thread pool coordinates a group of threads to run
/// taks in parallel.
///
/// # Example
/// ```
/// use job_pool::ThreadPool;
///
/// let pool = ThreadPool::with_size(32).expect("Error creating pool");
/// pool.execute(|| println!("Hello world!"));
/// ```
pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
    semaphore: Semaphore,
    max_jobs: Option<u16>,
}

impl ThreadPool {
    /// Create a new ThreadPool.
    ///
    /// # Returns
    /// A [Result]<[ThreadPool],[PoolError](crate::PoolError)>
    ///
    pub fn new(config: PoolConfig) -> Result<ThreadPool> {
        config.validate()?;
        let size = config.n_workers as usize;
        let (sender,receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        let semaphore = Arc::new((Mutex::new(0),Condvar::new()));
        let mut workers = Vec::with_capacity(size);
        for id in 0..size {
            let worker = Worker::new(id as u16,
                                    receiver.clone(),
                                    semaphore.clone());
            workers.push(worker);
        }
        Ok(ThreadPool {
            workers, semaphore,
            sender:Some(sender),
            max_jobs: config.max_jobs
        })
    }
    /// Create a [ThreadPool] with the default [configuration](PoolConfig)
    #[inline]
    pub fn default_config() -> Result<Self> {
        Self::new(PoolConfig::default())
    }
    /// Create a [ThreadPool] with a given size
    #[inline]
    pub fn with_size(size: u16) -> Result<Self> {
        Self::new(PoolConfig::with_size(size))
    }
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static
    {
        let (lock,cvar) = &*self.semaphore;
        let mut counter = lock.lock().unwrap();
        if let Some(max) = self.max_jobs {
            counter = cvar.wait_while(counter, |n| *n >= max).unwrap();
        }
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
