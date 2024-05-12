//! Thread Pool
//!
//! This crate contains code to run a Job pool.
//!
//! # Example
//! ```
//! use pool::ThreadPool;
//! use std::thread;
//! use std::time::Duration;
//!
//! let pool = ThreadPool::new(1024);
//! for _ in 0..10 {
//!     pool.execute(|| {
//!         thread::sleep(Duration::from_secs(5));
//!     });
//! }
//! pool.join();
//! ```

pub mod pool;
pub mod worker;
pub mod error;
use std::sync::{Arc, Condvar, Mutex};

pub use error::PoolError;
pub use pool::ThreadPool;

type Semaphore = Arc<(Mutex<u16>,Condvar)>;

pub type Result<T> = std::result::Result<T,PoolError>;
