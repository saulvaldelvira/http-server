use std::sync::mpsc::SendError;
use std::sync::{mpsc, Arc, Condvar, Mutex};
use crate::worker::{Job, Worker};
use crate::{PoolConfig, Result, Semaphore};

pub enum SenderWrapper<T> {
    Bounded(mpsc::SyncSender<T>),
    Unbounded(mpsc::Sender<T>),
}

impl<T> SenderWrapper<T> {
    fn send(&self, t: T) -> std::result::Result<(),SendError<T>> {
        match self {
            SenderWrapper::Bounded(b) => b.send(t),
            SenderWrapper::Unbounded(u) => u.send(t),
        }
    }
}

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
    sender: Option<SenderWrapper<Box<dyn Job>>>,
    semaphore: Semaphore,
    max_jobs: Option<u16>,
}

impl ThreadPool {
    /// Create a new ThreadPool.
    pub fn new(config: PoolConfig) -> Result<ThreadPool> {
        let size = config.n_workers as usize;
        let (sender,receiver) =
            if let Some(max) = config.incoming_buf_size {
                let (sender,receiver) = mpsc::sync_channel(max as usize);
                let sender = SenderWrapper::Bounded(sender);
                (sender,receiver)
            } else {
                let (sender,receiver) = mpsc::channel();
                let sender = SenderWrapper::Unbounded(sender);
                (sender,receiver)
            };
        let receiver = Arc::new(Mutex::new(receiver));
        let semaphore = Arc::new((Mutex::new(0),Condvar::new()));
        let mut workers = Vec::with_capacity(size);
        for _ in 0..size {
            let worker = Worker::new(receiver.clone(),semaphore.clone());
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
    pub fn with_default_config() -> Result<Self> {
        let conf = PoolConfig::builder()
                              .build().map_err(|err| err.to_string())?;
        Self::new(conf)
    }
    /// Create a [ThreadPool] with a given size
    #[inline]
    pub fn with_size(size: u16) -> Result<Self> {
        let conf = PoolConfig::builder()
                              .n_workers(size)
                              .build().map_err(|err| err.to_string())?;
        Self::new(conf)
    }
    pub fn execute(&self, job: impl Job) {
        {
            let (lock,cvar) = &*self.semaphore;
            let mut counter = lock.lock().unwrap();
            if let Some(max) = self.max_jobs {
                counter = cvar.wait_while(counter, |n| *n >= max).unwrap();
            }
            *counter += 1;
        }
        let job = Box::new(job);
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
