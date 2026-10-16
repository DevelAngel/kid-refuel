use thiserror::Error;

#[derive(Debug, Error)]
pub enum DatabaseError {
    #[error("DATABASE_URL is not available: {0}")]
    DatabaseUrl(#[from] std::env::VarError),
    #[error("failed to connect to the database: {0}")]
    Connection(#[from] diesel::ConnectionError),
    #[error("database operation failed: {0}")]
    Diesel(#[from] diesel::result::Error),
    #[error("database did not return an identifier for {0}")]
    MissingId(&'static str),
    #[error("database returned an invalid price: {0}")]
    InvalidPrice(i32),
}
