// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::workers::snapshot::error::Error;

use log::{info, warn};

use std::path::Path;

pub(crate) async fn download_snapshot_file(
    path: &Path,
    download_urls: impl Iterator<Item = &str>,
) -> Result<(), Error> {
    tokio::fs::create_dir_all(
        path.parent()
            .ok_or_else(|| Error::InvalidFilePath(path.to_string_lossy().to_string()))?,
    )
    .await
    .map_err(|_| Error::InvalidFilePath(path.to_string_lossy().to_string()))?;

    for url in download_urls {
        info!("Downloading snapshot file {}...", url);

        match reqwest::get(url).await.and_then(|res| res.error_for_status()) {
            Ok(res) => {
                tokio::io::copy(
                    &mut res.bytes().await.map_err(|_| Error::DownloadingFailed)?.as_ref(),
                    &mut tokio::fs::File::create(path).await?,
                )
                .await?;
                break;
            }
            Err(e) => warn!("Downloading snapshot file failed with status code {:?}.", e.status()),
        }
    }

    if !path.exists() {
        return Err(Error::NoDownloadSourceAvailable);
    }

    Ok(())
}
