//! Thread Pool
//!
//! This crate contains code to run a Job pool.
//!
//! # Example
//! ```rust,no_run
//! use job_pool::ThreadPool;
//! use std::thread;
//! use std::time::Duration;
//!
//! let pool = ThreadPool::new(1024).unwrap();
//! for _ in 0..10 {
//!     pool.execute(|| {
//!         thread::sleep(Duration::from_secs(5));
//!     });
//! }
//! pool.join();
//! ```

mod pool;
mod worker;
mod error;
use std::sync::{Arc, Condvar, Mutex};

pub use error::PoolError;
pub use pool::ThreadPool;

type Semaphore = Arc<(Mutex<u16>,Condvar)>;

pub type Result<T> = std::result::Result<T,PoolError>;
