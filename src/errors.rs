use actix_web::{http::StatusCode, HttpResponse, ResponseError};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Provided data could not be validated")]
    ValidationError(#[from] validator::ValidationErrors),

    #[error("The given password could not be hashed")]
    PasshwordHashError(argon2::password_hash::Error),

    #[error("The database query failed")]
    DatabaseError(#[from] diesel::result::Error),

    #[error("Could not [de]serialize data")]
    SerializationError(#[from] serde_json::Error),

    #[error("Could not [en|de]code data for token")]
    TokenError(#[from] branca::errors::Error),

    #[error("Failed while processing the request")]
    WebServerError(#[from] actix_web::Error),
}

impl From<argon2::password_hash::Error> for Error {
    fn from(value: argon2::password_hash::Error) -> Self {
        Self::PasshwordHashError(value)
    }
}

impl ResponseError for Error {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::ValidationError(_) => StatusCode::UNPROCESSABLE_ENTITY,
            Self::PasshwordHashError(_) => StatusCode::UNPROCESSABLE_ENTITY,
            Self::SerializationError(_) => StatusCode::UNPROCESSABLE_ENTITY,
            Self::TokenError(_) => StatusCode::UNPROCESSABLE_ENTITY,
            _ => StatusCode::INTERNAL_SERVER_ERROR
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}
