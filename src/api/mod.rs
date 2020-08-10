use super::Context;
use std::sync::Arc;
mod health;

use uuid::Uuid;
use warp::Filter;

pub struct Api;

pub struct Token(pub String);

#[derive(Debug)]
pub enum TokenParseError {
    Type,
}

impl std::str::FromStr for Token {
    type Err = TokenParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with("Bearer ") {
            let s = &s[7..];
            Ok(Token(s.to_string()))
        } else {
            Err(TokenParseError::Type)
        }
    }
}

impl Api {
    pub async fn serve(ctx: Context, addr: impl Into<std::net::SocketAddr> + 'static) {
        let ctx = Arc::new(ctx);
        let ctx = warp::any().map(move || ctx.clone());

        let validate_token = ctx
            .clone()
            .and(warp::header::<Token>("Authorization"))
            .map(|ctx: Arc<Context>, token: Token| ctx.tokens.validate_token(&token.0).unwrap());

        let prefix = warp::path!("api" / ..);

        let health = warp::path!("health").map(|| {
            let status = health::HealthCheckResponse::ok();
            warp::reply::json(&status)
        });

        let sessions_id = warp::path!("app" / String / "sessions" / Uuid)
            .and(ctx.clone())
            .and(validate_token.clone())
            .and_then(
                |app_id: String, session_id, ctx: Arc<Context>, tok| async move {
                    ctx.get_session(&app_id, session_id, tok)
                        .await
                        .map(|s| warp::reply::json(&s))
                        .map_err(|e| warp::reject::custom(e))
                },
            );

        let session_field = warp::path!("app" / String / "sessions" / Uuid / String)
            .and(ctx.clone())
            .and(validate_token.clone())
            .and_then(
                |app_id: String, session_id, field: String, ctx: Arc<Context>, tok| async move {
                    ctx.get_session_field(&app_id, session_id, field.as_ref(), tok)
                        .await
                        .map(|s| warp::reply::json(&s))
                        .map_err(|e| warp::reject::custom(e))
                },
            );

        let replace_session = warp::put()
            .and(warp::path!("app" / String / "sessions" / Uuid))
            .and(ctx.clone())
            .and(validate_token.clone())
            .and(warp::body::json::<serde_json::Value>())
            .and_then(
                |app_id: String, session_id, ctx: Arc<Context>, tok, body| async move {
                    ctx.replace_session(&app_id, session_id, body, tok)
                        .await
                        .map(|s| warp::reply::json(&s))
                        .map_err(|e| warp::reject::custom(e))
                },
            );

        let add_session = warp::post()
            .and(warp::path!("app" / String / "sessions"))
            .and(ctx.clone())
            .and(validate_token.clone())
            .and(warp::body::json::<serde_json::Value>())
            .and_then(|app_id: String, ctx: Arc<Context>, tok, body| async move {
                ctx.new_session(&app_id, body, tok)
                    .await
                    .map(|s| warp::reply::json(&s))
                    .map_err(|e| warp::reject::custom(e))
            });

        let api = prefix
            .and(warp::path!("v1" / ..))
            .and(
                health
                    .or(session_field)
                    .or(sessions_id)
                    .or(replace_session)
                    .or(add_session),
            )
            .recover(super::error::handle_error);

        warp::serve(api).run(addr).await;
    }
}
