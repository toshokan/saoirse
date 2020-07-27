use super::Context;
use std::sync::Arc;
mod health;

use uuid::Uuid;
use warp::Filter;

pub struct Api;

#[derive(Debug)]
pub struct Token(pub Uuid);

#[derive(Debug)]
pub enum TokenParseError {
    Type,
    Format,
}

impl std::str::FromStr for Token {
    type Err = TokenParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with("Saoirse ") {
            let s = &s[8..];
            Uuid::from_str(s)
                .map(|u| Token(u))
                .map_err(|_| TokenParseError::Format)
        } else {
            Err(TokenParseError::Type)
        }
    }
}

impl Api {
    pub async fn serve(ctx: Context, addr: impl Into<std::net::SocketAddr> + 'static) {
        let ctx = Arc::new(ctx);
        let ctx = warp::any().map(move || ctx.clone());

        let prefix = warp::path!("api" / ..);

        let health = warp::path!("health").map(|| {
            let status = health::HealthCheckResponse::ok();
            warp::reply::json(&status)
        });

        let sessions_id = warp::path!("app" / Uuid / "sessions" / Uuid)
            .and(ctx.clone())
            .and(warp::header::<Token>("Authorization"))
            .and_then(|app_id, session_id, ctx: Arc<Context>, tok| async move {
                ctx.get_session(app_id, session_id, tok)
                    .await
                    .map(|s| warp::reply::json(&s))
                    .map_err(|e| warp::reject::custom(e))
            });

        let session_field = warp::path!("app" / Uuid / "sessions" / Uuid / String)
            .and(ctx.clone())
            .and(warp::header::<Token>("Authorization"))
            .and_then(
                |app_id, session_id, field: String, ctx: Arc<Context>, tok| async move {
                    ctx.get_session_field(app_id, session_id, field.as_ref(), tok)
                        .await
                        .map(|s| warp::reply::json(&s))
                        .map_err(|e| warp::reject::custom(e))
                },
            );

        let create_app = warp::post()
            .and(warp::path!("app" / String))
            .and(ctx.clone())
            .and(warp::header::<Token>("Authorization"))
            .and_then(|name: String, ctx: Arc<Context>, tok| async move {
                ctx.create_app(&name, tok)
                    .await
                    .map(|s| warp::reply::json(&s))
                    .map_err(|e| warp::reject::custom(e))
            });

        let api = prefix
            .and(warp::path!("v1" / ..))
            .and(health.or(session_field).or(sessions_id).or(create_app))
            .recover(super::error::handle_error);

        warp::serve(api).run(addr).await;
    }
}
