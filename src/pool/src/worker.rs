use std::thread::{JoinHandle,spawn};
use std::sync::{mpsc, Arc, Mutex};
use crate::Semaphore;

/// Type of function ran by the [Worker]
pub trait Job: FnOnce() + Send + 'static {}
impl<T> Job for T
where T: FnOnce() + Send + 'static {}

/// Worker for the [ThreadPool](crate::ThreadPool)
pub struct Worker(Option<JoinHandle<()>>);

impl Worker {
    /// Creates a new [Worker]
    pub fn new(
        receiver: Arc<Mutex<mpsc::Receiver<Box<dyn Job>>>>,
        semaphore: Semaphore,
    ) -> Worker {
        let thread = spawn(move || loop {
            let message = receiver
                .lock()
                .unwrap()
                .recv();

            match message {
                Ok(job) => {
                    job();
                    let (lock,condv) = &*semaphore;
                    let mut counter = lock.lock().unwrap();
                    *counter -= 1;
                    condv.notify_one();
                }
                Err(_) => break,
            }
        });
        Worker(Some(thread))
    }
    /// Shuts down the [Worker]
    pub fn shutdown(&mut self) {
        if let Some(thread) = self.0.take() {
            thread.join().unwrap();
        }
    }
}
