use thiserror::Error;

use warp::{reject::Reject, Rejection, Reply};

#[derive(Error, Debug, serde::Serialize)]
#[serde(untagged)]
pub enum Error {
    #[error("Database error")]
    DbError(
        #[from]
        #[serde(skip)]
        sqlx::Error,
    ),
    #[error(transparent)]
    Session(#[from] super::SessionError),
    #[error(transparent)]
    Token(#[from] super::TokenError),
}

impl Reject for Error {}

#[derive(serde::Serialize)]
struct ErrorMessage<'e> {
    error: &'e Error,
    #[serde(skip_serializing_if = "Option::is_none")]
    info: Option<String>,
}

pub async fn handle_error(err: Rejection) -> Result<impl Reply, Rejection> {
    if let Some(e) = err.find::<Error>() {
        let message = ErrorMessage {
            error: e,
            info: None,
        };
        let reply = warp::reply::json(&message);
        return Ok(warp::reply::with_status(
            reply,
            warp::http::StatusCode::BAD_REQUEST,
        ));
    }

    Err(err)
}
