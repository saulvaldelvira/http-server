//! Thread Pool
//!
//! This crate contains code to run a Job pool.
//!
//! # Example
//! ```rust,no_run
//! use job_pool::*;
//! use std::thread;
//! use std::time::Duration;
//!
//! let conf = PoolConfig::default();
//! let pool = ThreadPool::new(conf).unwrap();
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
mod config;
use std::sync::{Arc, Condvar, Mutex};

pub use error::PoolError;
pub use pool::ThreadPool;

pub use config::PoolConfig;

type Semaphore = Arc<(Mutex<u16>,Condvar)>;

pub type Result<T> = std::result::Result<T,PoolError>;
