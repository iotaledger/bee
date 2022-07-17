// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use axum::{
    async_trait,
    extract::{FromRequest, RequestParts},
    BoxError,
};
use serde::de::DeserializeOwned;

use crate::error::{ApiError, DependencyError};

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
            Err(e) => Err(ApiError::DependencyError(DependencyError::AxumJsonError(e))),
        }
    }
}
