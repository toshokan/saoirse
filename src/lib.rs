pub mod api;
pub mod db;
pub mod error;

use jsonwebtoken::DecodingKey;
use sqlx::postgres::PgPool;
use std::env;
use uuid::Uuid;

pub struct Context {
    pool: sqlx::postgres::PgPool,
    tokens: TokenService,
}

#[derive(Debug, serde::Serialize)]
pub struct Session {
    id: Uuid,
    app_id: String,
    data: serde_json::Value,
}

#[derive(Debug, thiserror::Error, serde::Serialize)]
pub enum SessionError {
    #[error("Session attribute not present")]
    AttributeNotPresent,
}

#[derive(Debug, thiserror::Error, serde::Serialize)]
pub enum TokenError {
    #[error("Bad app token")]
    BadAppToken,
}

struct TokenService {
    scope: String,
    key: DecodingKey<'static>,
}

#[derive(serde::Deserialize)]
pub struct TokenClaims {
    sub: String,
    scope: String,
}

impl TokenService {
    fn new() -> Self {
        use std::io::Read;

        let scope = env::var("SAOIRSE_SCOPE").expect("Failed to get SAOIRSE_SCOPE");
        let jwt_key_path = env::var("JWT_PUBLIC_KEY").expect("Failed to get JWT_PUBLIC_KEY");
        let mut public_key_contents = vec![];
        std::fs::File::open(&jwt_key_path)
            .expect("Failed to open s key file")
            .read_to_end(&mut public_key_contents)
            .expect("Failed to read secret key file");

        let key = DecodingKey::from_ec_pem(&public_key_contents)
            .expect("Failed to parse public key")
            .into_static();

        Self { scope, key }
    }

    pub fn validate_token(&self, token: &str) -> Result<TokenClaims, ()> {
        use jsonwebtoken::{decode, TokenData, Validation};

        decode(token, &self.key, &Validation::default())
            .map(|t: TokenData<TokenClaims>| t.claims)
            .map_err(|_| {
                eprintln!("Bad token");
                ()
            })
            .and_then(|claims| {
                let split: Vec<String> = claims.scope.split(',').map(ToString::to_string).collect();
                if split.contains(&self.scope) {
                    Ok(claims)
                } else {
                    Err(())
                }
            })
    }
}

impl Context {
    pub async fn new() -> Result<Self, error::Error> {
        let pool = PgPool::builder()
            .max_size(10)
            .build(&env::var("DATABASE_URL").expect("Failed to get DATABASE_URL"))
            .await?;

        let tokens = TokenService::new();

        Ok(Self { pool, tokens })
    }

    pub async fn get_session(
        &self,
        app_id: &str,
        id: Uuid,
        app_token: TokenClaims,
    ) -> Result<Option<Session>, error::Error> {
        // self.check_app_token(&app_id, &app_token.0).await?; // TODO

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
        app_id: &str,
        id: Uuid,
        name: &str,
        app_token: TokenClaims,
    ) -> Result<serde_json::Value, error::Error> {
        // self.check_app_token(&app_id, &app_token.0).await?; // TODO

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

    pub async fn replace_session(
        &self,
        app_id: &str,
        id: Uuid,
        body: serde_json::Value,
        app_token: TokenClaims,
    ) -> Result<Session, error::Error> {
        // self.check_app_token(&app_id, &app_token.0).await?;

        let session = sqlx::query_as!(
	    Session,
	    "UPDATE sessions SET data = $1 WHERE session_id = $2 AND app_id = $3 RETURNING session_id AS id, app_id, data",
	    body,
	    id,
	    app_id
	)
	    .fetch_one(&self.pool)
	    .await?;

        Ok(session)
    }

    pub async fn new_session(
        &self,
        app_id: &str,
        body: serde_json::Value,
        app_token: TokenClaims,
    ) -> Result<Session, error::Error> {
        // self.check_app_token(&app_id, &app_token.0).await?;

        let session = sqlx::query_as!(
	    Session,
	    "INSERT INTO sessions(app_id, data) VALUES($1, $2) RETURNING session_id as id, app_id, data",
	    app_id,
	    body
	)
        .fetch_one(&self.pool)
        .await?;

        Ok(session)
    }
}
