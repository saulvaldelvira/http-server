use builders::Builder;
use crate::Result;

/// Pool Config
///
/// Configuration for the [ThreadPool](crate::ThreadPool)
#[derive(Clone,Copy,Builder)]
pub struct PoolConfig {
    #[builder(def = 1024_u16)]
    pub n_workers: u16,
    #[builder(optional = true)]
    pub max_jobs: Option<u16>,
    /// Default value: None
    #[builder(def = { None } )]
    pub incoming_buf_size: Option<u16>,
}

impl PoolConfig {
    pub fn validate(&self) -> Result<()> {
        if self.n_workers == 0 {
            return Err("Invalid pool size: 0".into());
        }
        if let Some(max) = self.max_jobs {
            if max < self.n_workers {
                return Err(format!("Max number of jobs ({max}) is lower \
                        than the number of workers ({})", self.n_workers).into())
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
    /// Incoming buf size: 4096
    fn default() -> Self {
        PoolConfig::builder().build().unwrap()
    }
}
