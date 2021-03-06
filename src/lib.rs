#[macro_use] extern crate log;
extern crate iron;
extern crate r2d2_postgres;

use iron::prelude::*;
use iron::{typemap, BeforeMiddleware};

use std::error::Error;
use std::sync::Arc;
use r2d2_postgres::{TlsMode, PostgresConnectionManager, r2d2};

type PPool       = r2d2::Pool<r2d2_postgres::PostgresConnectionManager>             ;
type PConnection = r2d2::PooledConnection<r2d2_postgres::PostgresConnectionManager> ;

/// Iron middleware that allows for postgres connections within requests.
#[derive(Debug)]
pub struct PostgresMiddleware {
  /// A pool of postgres connections that are shared between requests.
  //pub pool: Arc<r2d2::Pool<r2d2_postgres::PostgresConnectionManager>> ,
  pub pool: Arc<PPool> ,
}

#[derive(Debug)]
pub struct Value(Arc<PPool>);

impl typemap::Key for PostgresMiddleware { type Value = Value; }

impl PostgresMiddleware {

  /// Creates a new pooled connection to the given postgresql server. The URL is in the format:
  ///
  /// ```{none}
  /// postgresql://user[:password]@host[:port][/database][?param1=val1[[&param2=val2]...]]
  /// ```
  ///
  /// Returns `Err(err)` if there are any errors connecting to the postgresql database.
  ///
    /*
  pub fn new(pg_connection_str: &str) -> Result<PostgresMiddleware, Box<Error>> {
    let config = r2d2::Config::builder()
        .error_handler(Box::new(r2d2::LoggingErrorHandler))
        .build();
    let manager = try!(PostgresConnectionManager::new(pg_connection_str, TlsMode::None));
    let pool = Arc::new(try!(r2d2::Pool::new(config, manager)));
    Ok(PostgresMiddleware {
      pool: pool,
    })
  }
  */

  pub fn new(pg_connection_str: &str, pool_size: u32 ) -> Result<PostgresMiddleware, Box<Error>> {
    debug!("201808112142 PostgresMiddleware::new() connecting to database at {}", pg_connection_str);
    let manager = PostgresConnectionManager::new(pg_connection_str, TlsMode::None).unwrap() ;
    let pool  = r2d2::Pool::builder()
      .max_size(pool_size)
      .build(manager)
      .unwrap() 
      ;
    debug!("201808120931 PostgresMiddleware::new() connected to database pool= \n{:?}" , pool );
    let pool =  Arc::new(pool);
    Ok(PostgresMiddleware { pool: pool, })
  }
}

impl BeforeMiddleware for PostgresMiddleware {
    fn before(&self, req: &mut Request) -> IronResult<()> {
        req.extensions.insert::<PostgresMiddleware>(Value(self.pool.clone()));
        Ok(())
    }
}

/// Adds a method to requests to get a database connection.
///
/// ## Example
///
/// ```ignore
/// fn handler(req: &mut Request) -> IronResult<Response> {
///   let conn = req.db_conn();
///   con.execute("INSERT INTO foo (bar) VALUES ($1)", &[&1i32]).unwrap();
///
///   Ok(Response::with((status::Ok, resp_str)))
/// }
/// ```
pub trait PostgresReqExt {
  /// Returns a pooled connection to the postgresql database. The connection is returned to
  /// the pool when the pooled connection is dropped.
  ///
  /// **Panics** if a `PostgresMiddleware` has not been registered with Iron, or if retrieving
  /// a connection to the database times out.
  fn db_conn(&self) -> PConnection ;
}

impl<'a, 'b> PostgresReqExt for Request<'a, 'b> {
  fn db_conn(&self) -> PConnection {
    let poll_value = self.extensions.get::<PostgresMiddleware>().unwrap();
    let &Value(ref poll) = poll_value;
    debug!("201808122156 PostgresReqExt::db_conn  poll = {:?}" , poll);
    let db_conn = poll.get().unwrap();
    debug!("201808120958 PostgresReqExt::db_conn  db_conn = {:?}" , db_conn);
    return db_conn;
  }
}
