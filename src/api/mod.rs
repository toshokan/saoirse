use super::Ctx;
mod health;

use warp::Filter;

pub struct Api<'c> {
    ctx: &'c Ctx
}

impl<'c> Api<'c> {
    pub fn new(ctx: &'c Ctx) -> Self {
	Self {
	    ctx
	}
    }
    
    pub async fn serve(&self, addr: impl Into<std::net::SocketAddr> + 'static) {
	let prefix = warp::path!("api" / ..);
	
	let health = warp::path!("health")
	    .map(|| {
		let status = health::HealthCheckResponse::ok();
		warp::reply::json(&status)
	    });

	let api = prefix
	    .and(warp::path!("v1"/ .. ))
	    .and(health);

	warp::serve(api)
	    .run(addr)
	    .await;
    }
}
