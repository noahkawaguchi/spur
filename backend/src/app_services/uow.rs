use crate::repository::error::RepoError;
use sqlx::{PgConnection, PgExecutor, PgPool, PgTransaction};

#[async_trait::async_trait]
pub trait Tx: Send {
    fn exec(&mut self) -> impl PgExecutor<'_>;
    async fn commit_uow(self) -> Result<(), RepoError>;
}

#[async_trait::async_trait]
pub trait UnitOfWork: Send + Sync {
    type Tx<'c>: Tx;
    async fn begin_uow<'c>(&self) -> Result<Self::Tx<'c>, RepoError>;
    fn single_exec(&self) -> impl PgExecutor<'_>;
}

#[async_trait::async_trait]
impl Tx for PgTransaction<'_> {
    fn exec(&mut self) -> impl PgExecutor<'_> {
        let exec: &mut PgConnection = &mut *self.as_mut();
        exec
    }

    async fn commit_uow(self) -> Result<(), RepoError> { self.commit().await.map_err(Into::into) }
}

#[async_trait::async_trait]
impl UnitOfWork for PgPool {
    type Tx<'c> = PgTransaction<'c>;

    async fn begin_uow<'c>(&self) -> Result<Self::Tx<'c>, RepoError> {
        self.begin().await.map_err(Into::into)
    }

    fn single_exec(&self) -> impl PgExecutor<'_> { self }
}
