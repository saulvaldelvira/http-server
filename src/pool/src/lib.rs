pub mod pool;
pub mod worker;
pub mod error;
use std::sync::{Arc, Condvar, Mutex};

pub use error::PoolError;
pub use pool::ThreadPool;

type Semaphore = Arc<(Mutex<u16>,Condvar)>;

pub type Result<T> = std::result::Result<T,PoolError>;
