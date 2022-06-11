use diesel::PgConnection;
use r2d2::{CustomizeConnection, Pool, PooledConnection};
use r2d2_diesel::ConnectionManager;

pub type MyDBConnectionManager = ConnectionManager<PgConnection>;
pub type MyPool = Pool<MyDBConnectionManager>;
#[allow(dead_code)]
pub type MyDBConnection = PooledConnection<MyDBConnectionManager>;

#[derive(Debug)]
pub struct MyDBConnectionCustomizer { }

impl MyDBConnectionCustomizer {
    pub fn new() -> Self {
        Self { }
    }
}

impl CustomizeConnection<PgConnection, r2d2_diesel::Error> for MyDBConnectionCustomizer {
    fn on_acquire(&self, _conn: &mut PgConnection) -> Result<(), r2d2_diesel::Error> { // TODO: Less wide error.
        // sql_query("SET SESSION statement_timeout = 2000").execute(conn).map_err(|err| r2d2_diesel::Error::QueryError(err))?; // 2 sec
        Ok(())
    }
}

pub fn db_pool_builder() -> r2d2::Builder<MyDBConnectionManager> {
    r2d2::Pool::builder()
}
