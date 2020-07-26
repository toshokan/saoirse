pub mod api;
pub mod error;

use std::env;
use sqlx::postgres::PgPool;

pub struct Context {
    pool: sqlx::postgres::PgPool
}

impl Context {
    pub async fn new() -> Result<Self, error::Error> {
	let pool = PgPool::builder()
            .max_size(10)
            .build(&env::var("DATABASE_URL").expect("Failed to get DATABASE_URL"))
            .await?;

	Ok(Self {
	    pool
	})
    }
}
