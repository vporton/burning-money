// use diesel::PgConnection;
//
// #[derive(Debug)]
// pub struct MyDBConnectionCustomizer { }
//
// impl MyDBConnectionCustomizer {
//     pub fn new() -> Self {
//         Self { }
//     }
// }
//
// impl CustomizeConnection<PgConnection, MyError> for MyDBConnectionCustomizer {
//     fn on_acquire(&self, _conn: &mut PgConnection) -> Result<(), MyError> { // TODO: Less wide error.
//         // sql_query("SET SESSION statement_timeout = 2000").execute(conn).map_err(|err| r2d2_diesel::Error::QueryError(err))?; // 2 sec
//         Ok(())
//     }
// }
