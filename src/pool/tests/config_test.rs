use job_pool::PoolConfig;

#[test]
fn blocking_under() {
    let config = PoolConfig::builder()
                            .n_workers(10_u16)
                            .max_jobs(5_u16)
                            .build()
                            .unwrap();
    match config.validate() {
        Ok(_) => panic!("Expected Err value"),
        Err(err) => assert_eq!("Max number of jobs (5) is lower \
                                than the number of workers (10)", err.to_string())
    }
}

#[test]
fn size_0() {
    let config = PoolConfig::builder()
                            .n_workers(0_u16)
                            .build()
                            .unwrap();
    match config.validate() {
        Ok(_) => panic!("Expected Err value"),
        Err(err) => assert_eq!("Invalid pool size: 0", err.to_string())
    }
}
