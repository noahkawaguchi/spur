use crate::{
    app_services::uow::{Tx, UnitOfWork},
    domain::RepoError,
};
use anyhow::{Context, Result};
use sqlx::{PgExecutor, PgPool, postgres::PgPoolOptions};
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering::SeqCst},
    },
    time::Duration,
};

/// Creates a fake `PgPool` for tests in which one is required but never used. Simpler than mocking
/// a unit of work if transactions will not be used in the test.
pub fn fake_pool() -> Result<PgPool> {
    PgPoolOptions::new()
        // Fail fast if something attempts to actually hit the DB
        .acquire_timeout(Duration::from_millis(50))
        // Must look like a URL
        .connect_lazy("postgres://user:pass@127.0.0.1:1/not_real")
        .context("lazy pool should always build")
}

/// Probe for checking if a fake transaction was committed.
#[derive(Clone)]
pub struct CommitProbe(Arc<AtomicBool>);
impl CommitProbe {
    pub fn commit_called(&self) -> bool { self.0.load(SeqCst) }
}

#[derive(Clone)]
pub struct FakeTx {
    pool: PgPool,
    probe: CommitProbe,
}

/// A fake unit of work struct for testing purposes. Beginning and committing transactions will
/// compile and run, but have no effect on any real or temporary database. However, attempting to
/// read or write data will fail.
pub struct FakeUow {
    fake_tx: FakeTx,
}

impl FakeUow {
    pub fn with_probe() -> Result<(Self, CommitProbe)> {
        let probe = CommitProbe(Arc::new(AtomicBool::new(false)));
        let fake_tx = FakeTx { pool: fake_pool()?, probe: probe.clone() };
        Ok((Self { fake_tx }, probe))
    }
}

#[async_trait::async_trait]
impl Tx for FakeTx {
    async fn commit_uow(self) -> Result<(), RepoError> {
        self.probe.0.store(true, SeqCst);
        Ok(())
    }

    fn exec(&mut self) -> impl PgExecutor<'_> { &self.pool }
}

#[async_trait::async_trait]
impl UnitOfWork for FakeUow {
    type Tx<'c> = FakeTx;

    async fn begin_uow<'c>(&self) -> Result<Self::Tx<'c>, RepoError> { Ok(self.fake_tx.clone()) }

    fn single_exec(&self) -> impl PgExecutor<'_> { &self.fake_tx.pool }
}
