use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Database error")]
    Sqlx(#[from] sqlx::Error)
}
