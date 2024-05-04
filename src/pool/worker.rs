use std::thread::{JoinHandle,spawn};
use std::sync::{mpsc, Arc, Mutex};
use super::Semaphore;

/// Type of function ran by the [Worker]
pub type Job = Box<dyn FnOnce() + Send + 'static>;

/// Worker for the [ThreadPool]
pub struct Worker {
    id: usize,
    thread: Option<JoinHandle<()>>,
}

impl Worker {
    /// Creates a new [Worker]
    pub fn new(
        id: usize,
        receiver: Arc<Mutex<mpsc::Receiver<Job>>>,
        semaphore: Semaphore,
    ) -> Worker {
        let thread = spawn(move || loop {
            let message = receiver
                .lock()
                .unwrap()
                .recv();

            match message {
                Ok(job) => {
                    /* println!("Worker {id} got a job."); */
                    job();
                    let (lock,condv) = &*semaphore;
                    let mut counter = lock.lock().unwrap();
                    *counter -= 1;
                    condv.notify_one();
                }
                Err(_) => break,
            }
        });
        Worker { id,thread:Some(thread)}
    }
    /// Shuts down the [Worker]
    pub fn shutdown(&mut self) {
        println!("Shutting down worker {}", self.id);
        if let Some(thread) = self.thread.take() {
            thread.join().unwrap();
        }
    }
}

