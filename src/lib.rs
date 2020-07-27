pub mod api;
pub mod db;
pub mod error;

use sqlx::postgres::PgPool;
use std::env;
use uuid::Uuid;

pub struct Context {
    pool: sqlx::postgres::PgPool,
}

#[derive(Debug, serde::Serialize)]
pub struct Session {
    id: Uuid,
    app_id: Uuid,
    data: serde_json::Value,
}

#[derive(Debug, thiserror::Error, serde::Serialize)]
pub enum SessionError {
    #[error("Session attribute not present")]
    AttributeNotPresent,
}

#[derive(Debug, thiserror::Error, serde::Serialize)]
pub enum TokenError {
    #[error("Bad admin token")]
    BadAdminToken,
    #[error("Bad app token")]
    BadAppToken,
}

#[derive(Debug)]
#[derive(serde::Serialize)]
pub struct App {
    id: Uuid,
    name: String,
    token: Uuid,
}

impl Context {
    pub async fn new() -> Result<Self, error::Error> {
        let pool = PgPool::builder()
            .max_size(10)
            .build(&env::var("DATABASE_URL").expect("Failed to get DATABASE_URL"))
            .await?;

        Ok(Self { pool })
    }

    pub async fn check_admin_token(&self, token: &Uuid) -> Result<(), error::Error> {
        let token = sqlx::query!("SELECT * FROM admin_tokens WHERE token = $1", token.clone())
            .fetch_optional(&self.pool)
            .await?;

        if token.is_some() {
            Ok(())
        } else {
            Err(TokenError::BadAdminToken.into())
        }
    }

    pub async fn check_app_token(&self, app: &Uuid, token: &Uuid) -> Result<(), error::Error> {
        let token = sqlx::query!(
            "SELECT * FROM apps WHERE app_id = $1 AND token = $2",
            app.clone(),
            token.clone()
        )
        .fetch_optional(&self.pool)
        .await?;

        if token.is_some() {
            Ok(())
        } else {
            Err(TokenError::BadAppToken.into())
        }
    }

    pub async fn get_session(&self, app_id: Uuid, id: Uuid, app_token: api::Token) -> Result<Option<Session>, error::Error> {
	self.check_app_token(&app_id, &app_token.0).await?;
	
        let session = sqlx::query_as!(
            Session,
            "Select session_id AS id, app_id, data FROM sessions WHERE session_id = $1 AND app_id = $2",
            id,
	    app_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(session)
    }

    pub async fn get_session_field(
        &self,
	app_id: Uuid,
        id: Uuid,
        name: &str,
	app_token: api::Token
    ) -> Result<serde_json::Value, error::Error> {
	self.check_app_token(&app_id, &app_token.0).await?;
	
        let value = sqlx::query_as!(
            Session,
            "SELECT session_id as id, app_id, data FROM sessions WHERE session_id = $1 AND app_id = $2",
            id,
	    app_id
        )
        .fetch_optional(&self.pool)
        .await?
        .and_then(|s| s.data.get(name).cloned());

        value.ok_or_else(|| SessionError::AttributeNotPresent.into())
    }

    pub async fn create_app(&self, name: &str, admin_token: api::Token) -> Result<App, error::Error> {
        self.check_admin_token(&admin_token.0).await?;

        let app = sqlx::query_as!(
            App,
            "INSERT INTO apps (name) VALUES ($1) RETURNING app_id as id, name, token",
            name
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(app)
    }
}
