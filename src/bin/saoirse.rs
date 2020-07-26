use warp::Filter;

#[derive(serde::Serialize)]
pub enum HealthStatus {
    OK,
    Unavailable
}

#[derive(serde::Serialize)]
pub struct HealthCheckResponse {
    status: HealthStatus
}

impl HealthCheckResponse {
    fn ok() -> Self {
	Self {
	    status: HealthStatus::OK
	}
    }
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let prefix = warp::path!("api" / ..);
    
    let health = warp::path!("health")
	.map(|| {
	    let status = HealthCheckResponse::ok();
	    warp::reply::json(&status)
	});

    let api = prefix
	.and(warp::path!("v1"/ .. ))
	.and(health);

    warp::serve(api)
	.run(([127, 0, 0, 1], 8080))
	.await;
    Ok(())
}
