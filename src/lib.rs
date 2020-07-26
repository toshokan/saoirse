pub mod api;
pub mod error;
pub mod db;

use std::env;
use sqlx::postgres::PgPool;
use uuid::Uuid;

pub struct Context {
    pool: sqlx::postgres::PgPool
}

#[derive(Debug)]
pub struct Session {
    id: Uuid,
    app_id: Uuid,
    data: serde_json::Value
}

#[derive(Debug)]
pub struct App {
    id: Uuid,
    name: String,
    token: Uuid
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

    pub async fn check_admin_token(&self, token: &Uuid) -> Result<(), error::Error> {
	let token = sqlx::query!("SELECT * FROM admin_tokens WHERE token = $1", token.clone())
	    .fetch_optional(&self.pool)
	    .await?;
	
	if token.is_some() {
	    Ok(())
	} else {
	    panic!()
	}

    }

    pub async fn get_sessions(&self) -> Result<Vec<Session>, error::Error> {
	let sessions = sqlx::query_as!(Session, "SELECT session_id AS id, app_id, data FROM sessions")
	    .fetch_all(&self.pool)
	    .await?;
	
	Ok(sessions)
    }

    pub async fn create_app(&self, admin_token: Uuid, name: &str) -> Result<App, error::Error> {
	self.check_admin_token(&admin_token).await?;
	
	let app = sqlx::query_as!(App, "INSERT INTO apps (name) VALUES ($1) RETURNING app_id as id, name, token", name)
	    .fetch_one(&self.pool)
	    .await?;

	Ok(app)
    }
}
