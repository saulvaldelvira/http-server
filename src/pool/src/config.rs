use crate::{PoolError,Result};

/// Pool Config
///
/// Configuration for the [ThreadPool](crate::ThreadPool)
#[derive(Clone,Copy)]
pub struct PoolConfig {
    pub n_workers: u16,
    pub max_jobs: Option<u16>,
}

impl PoolConfig {
    /// Create a [PoolConfig] witb a given size
    #[inline]
    pub fn with_size(size: u16) -> Self {
        Self::default().n_workers(size)
    }
    /// Set the pool to be blocking when receiving a
    /// given number of jobs
    #[inline]
    pub fn max_jobs(mut self, n_jobs: u16) -> Self {
        self.max_jobs = Some(n_jobs);
        self
    }
    /// Set the pool to be blocking when no worker is
    /// available
    #[inline]
    pub fn blocking(mut self) -> Self {
        self.max_jobs = Some(self.n_workers);
        self
    }
    /// Set the pool to be non-blocking
    #[inline]
    pub fn non_blocking(mut self) -> Self {
        self.max_jobs = None;
        self
    }
    /// Set the number of workers
    #[inline]
    pub fn n_workers(mut self, n_workers: u16) -> Self {
        self.n_workers = n_workers;
        self
    }
    #[inline]
    pub fn is_blocking(&self) -> bool {
        self.max_jobs.is_some()
    }
    pub fn validate(&self) -> Result<()> {
        if self.n_workers == 0 {
            return PoolError::from_str("Invalid pool size: 0").err();
        }
        if let Some(max) = self.max_jobs {
            if max < self.n_workers {
                return PoolError::from_string(
                    format!("Max number of jobs ({max}) is lower \
                             than the number of workers ({})", self.n_workers)).err()
            }
        }
        Ok(())
    }
}

impl Default for PoolConfig {
    /// Default configuration
    ///
    /// Nº Workers: 1024
    /// Max Jobs: None
    #[inline]
    fn default() -> Self {
        Self {
            n_workers: 1024,
            max_jobs: None,
        }
    }
}
