// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::workers::snapshot::error::Error;

use log::{info, warn};

use std::{fs::File, io::copy, path::Path};

pub(crate) async fn download_snapshot_file(file_path: &Path, download_urls: &[String]) -> Result<(), Error> {
    let file_name = file_path
        .file_name()
        .ok_or_else(|| Error::InvalidFilePath(file_path.to_string_lossy().to_string()))?;

    tokio::fs::create_dir_all(
        file_path
            .parent()
            .ok_or_else(|| Error::InvalidFilePath(file_path.to_string_lossy().to_string()))?,
    )
    .await
    .map_err(|_| Error::InvalidFilePath(file_path.to_string_lossy().to_string()))?;

    for url in download_urls {
        let url = url.to_owned() + &file_name.to_string_lossy();

        info!("Downloading snapshot file {}...", url);

        match reqwest::get(&url).await.and_then(|res| res.error_for_status()) {
            Ok(res) => {
                copy(
                    &mut res.bytes().await.map_err(|_| Error::DownloadingFailed)?.as_ref(),
                    &mut File::create(file_path)?,
                )?;
                break;
            }
            Err(e) => warn!("Downloading snapshot file failed with status code {:?}.", e.status()),
        }
    }

    if !file_path.exists() {
        return Err(Error::NoDownloadSourceAvailable);
    }

    Ok(())
}
