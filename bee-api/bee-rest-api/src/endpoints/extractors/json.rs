// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use axum::{
    async_trait,
    extract::{rejection::JsonRejection, FromRequest, RequestParts},
    BoxError,
};
use log::error;
use serde::de::DeserializeOwned;

use crate::endpoints::error::ApiError;

// We define our own `Path` extractor that customizes the error from `axum::extract::Path`
pub struct CustomJson<T>(pub T);

#[async_trait]
impl<B, T> FromRequest<B> for CustomJson<T>
where
    // these trait bounds are copied from `impl FromRequest for axum::Json`
    T: DeserializeOwned,
    B: axum::body::HttpBody + Send,
    B::Data: Send,
    B::Error: Into<BoxError>,
{
    type Rejection = ApiError;

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        match axum::Json::<T>::from_request(req).await {
            Ok(value) => Ok(Self(value.0)),
            Err(rejection) => {
                // convert the error from `axum::Json` into whatever we want
                let err = match rejection {
                    JsonRejection::JsonDataError(err) => ApiError::InvalidJson(err.to_string()),
                    JsonRejection::MissingJsonContentType(err) => ApiError::InvalidJson(err.to_string()),
                    err => {
                        error!("unhandled JSON extractor error: {}", err);
                        ApiError::InternalError
                    }
                };

                Err(err)
            }
        }
    }
}
