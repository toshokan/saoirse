#[derive(serde::Serialize)]
pub enum HealthStatus {
    OK,
    Unavailable,
}

#[derive(serde::Serialize)]
pub struct HealthCheckResponse {
    status: HealthStatus,
}

impl HealthCheckResponse {
    pub fn ok() -> Self {
        Self {
            status: HealthStatus::OK,
        }
    }
}
