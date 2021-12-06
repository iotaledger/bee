// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{types::snapshot::SnapshotHeader, workers::snapshot::error::Error};

use bee_common::packable::Packable;

use bytes::Buf;
use futures::{future::join_all, StreamExt};
use log::{info, warn};

use std::{io::Read, path::Path};

async fn download_snapshot_header(download_url: &str) -> Result<(SnapshotHeader, &str), Error> {
    info!("Downloading snapshot header {}...", download_url);

    match reqwest::get(download_url).await.and_then(|res| res.error_for_status()) {
        Ok(res) => {
            let mut stream = res.bytes_stream();
            let mut bytes = Vec::<u8>::with_capacity(SnapshotHeader::LENGTH);

            while let Some(chunk) = stream.next().await {
                let mut chunk_reader = chunk.map_err(|_| Error::DownloadingFailed)?.reader();

                let mut buf = Vec::new();
                chunk_reader.read_to_end(&mut buf)?;
                bytes.extend(buf.iter());

                if bytes.len() >= SnapshotHeader::LENGTH {
                    info!("Downloaded snapshot header from {}.", download_url);

                    let mut slice: &[u8] = &bytes[..SnapshotHeader::LENGTH];

                    return Ok((SnapshotHeader::unpack(&mut slice)?, download_url));
                }
            }
        }
        Err(e) => warn!("Downloading snapshot header failed: {:?}.", e.to_string()),
    }

    Err(Error::DownloadingFailed)
}

async fn most_recent_snapshot_url<'a>(download_urls: impl Iterator<Item = &'a str>) -> Result<&'a str, Error> {
    let downloads = join_all(download_urls.map(download_snapshot_header)).await;
    let snapshot_headers = downloads.iter().flatten();

    // Find header with largest index.
    let largest_index = snapshot_headers.max_by_key(|(header, _)| header.ledger_index());

    // We know `snapshot_headers` is not empty, so unwrapping here is fine.
    if let Some((_, url)) = largest_index {
        Ok(url)
    } else {
        Err(Error::NoDownloadSourceAvailable)
    }
}

pub(crate) async fn download_snapshot_file(
    path: &Path,
    download_urls: impl Iterator<Item = &str>,
) -> Result<(), Error> {
    let download_url = most_recent_snapshot_url(download_urls).await?;

    tokio::fs::create_dir_all(
        path.parent()
            .ok_or_else(|| Error::InvalidFilePath(format!("{}", path.display())))?,
    )
    .await
    .map_err(|_| Error::InvalidFilePath(format!("{}", path.display())))?;

    info!("Downloading snapshot file {}...", download_url);

    match reqwest::get(download_url).await {
        Ok(res) => {
            tokio::io::copy(
                &mut res.bytes().await.map_err(|_| Error::DownloadingFailed)?.as_ref(),
                &mut tokio::fs::File::create(path).await?,
            )
            .await?;
        }
        Err(e) => warn!("Downloading snapshot file failed with status code {:?}.", e.status()),
    }

    if !path.exists() {
        return Err(Error::NoDownloadSourceAvailable);
    }

    Ok(())
}
