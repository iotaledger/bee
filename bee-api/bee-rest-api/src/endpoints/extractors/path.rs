// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use axum::{
    async_trait,
    extract::{FromRequest, RequestParts},
};
use serde::de::DeserializeOwned;

use crate::endpoints::error::{ApiError, DependencyError};

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
            Err(e) => Err(ApiError::DependencyError(DependencyError::InvalidPath(e))),
        }
    }
}
