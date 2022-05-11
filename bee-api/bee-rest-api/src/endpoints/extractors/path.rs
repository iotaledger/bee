//! Run with
//!
//! ```not_rust
//! cd examples && cargo run -p example-customize-path-rejection
//! ```
use axum::{
    async_trait,
    extract::{rejection::PathRejection, FromRequest, RequestParts},
};
use log::error;
use serde::de::DeserializeOwned;

use crate::endpoints::error::ApiError;

// We define our own `Path` extractor that customizes the error from `axum::extract::Path`
pub struct CustomPath<T>(pub T);

#[async_trait]
impl<B, T> FromRequest<B> for CustomPath<T>
where
    // these trait bounds are copied from `impl FromRequest for
    // axum::extract::path::Path`
    T: DeserializeOwned + Send,
    B: Send,
{
    type Rejection = ApiError;

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        match axum::extract::Path::<T>::from_request(req).await {
            Ok(value) => Ok(Self(value.0)),
            Err(rejection) => {
                let err = match rejection {
                    PathRejection::FailedToDeserializePathParams(inner) => {
                        ApiError::InvalidPathProvided(inner.into_kind().to_string())
                    }
                    PathRejection::MissingPathParams(error) => ApiError::InvalidPathProvided(error.to_string()),
                    _ => {
                        error!("unhandled path rejection: {}", rejection);
                        ApiError::InternalError
                    }
                };

                Err(err)
            }
        }
    }
}
