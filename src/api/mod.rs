use super::Context;
use std::sync::Arc;
mod health;

use warp::Filter;
use uuid::Uuid;

pub struct Api {
    ctx: Arc<Context>,
}

impl Api {
    pub fn new(ctx: Context) -> Self {
        Self { ctx: Arc::new(ctx) }
    }

    pub async fn serve(self, addr: impl Into<std::net::SocketAddr> + 'static) {
	let ctx = self.ctx;
	
        let with_context = warp::any().map(move || ctx.clone());

        let prefix = warp::path!("api" / ..);

        let health = warp::path!("health").map(|| {
            let status = health::HealthCheckResponse::ok();
            warp::reply::json(&status)
        });

        let sessions =
            warp::path!("sessions")
                .and(with_context.clone())
                .and_then(|ctx: Arc<Context>| async move {
                    ctx.get_sessions()
                        .await
                        .map(|s| warp::reply::json(&s))
                        .map_err(|e| warp::reject::custom(e))
                });

	let session_field =
	    warp::path!("sessions" / Uuid / String)
    	    .and(with_context)
    	    .and_then(|sid, field: String, ctx: Arc<Context>| async move {
		ctx.get_session_field(sid, field.as_ref())
		    .await
		    .map(|s| warp::reply::json(&s))
		    .map_err(|e| warp::reject::custom(e))
	    });

        let api = prefix.and(warp::path!("v1" / ..)).and(health.or(sessions).or(session_field)).recover(super::error::handle_error);

        warp::serve(api).run(addr).await;
    }
}
