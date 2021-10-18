// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{types::snapshot::SnapshotHeader, workers::snapshot::error::Error};

use bee_common::packable::Packable;

use bytes::Buf;
use futures::{future::join_all, StreamExt};
use log::{info, warn};

use std::{io::Read, path::Path};

pub(crate) async fn download_snapshot_file(
    path: &Path,
    download_urls: impl Iterator<Item = &str>,
) -> Result<(), Error> {
    let url = most_recent_snapshot_url(download_urls).await?;
    download_full_snapshot(path, url).await?;

    Ok(())
}

async fn most_recent_snapshot_url<'a>(download_urls: impl Iterator<Item = &'a str>) -> Result<&'a str, Error> {
    let downloads = join_all(download_urls.map(download_snapshot_header)).await;

    let snapshot_headers = downloads.into_iter().collect::<Result<Vec<_>, Error>>()?;

    let mut snapshot_headers = snapshot_headers.iter().flatten().collect::<Vec<_>>();

    if snapshot_headers.is_empty() {
        return Err(Error::NoDownloadSourceAvailable);
    }

    // Sort the headers so that the largest ledger index is in the last element.
    snapshot_headers.sort_by(|(header_a, _), (header_b, _)| header_a.ledger_index().cmp(&header_b.ledger_index()));

    // We know `snapshot_headers` is not empty, so unwrapping here is fine.
    let (_, url) = snapshot_headers.pop().unwrap();

    Ok(url)
}

async fn download_snapshot_header<'a>(download_url: &'a str) -> Result<Option<(SnapshotHeader, &'a str)>, Error> {
    info!("Downloading snapshot header {}...", download_url);

    match reqwest::get(download_url).await {
        Ok(res) => {
            let mut stream = res.bytes_stream();
            let mut bytes = Vec::<u8>::new();

            while let Some(chunk) = stream.next().await {
                let mut chunk_reader = chunk.unwrap().reader();

                let mut buf = Vec::new();
                chunk_reader.read_to_end(&mut buf)?;
                bytes.extend(buf.iter());

                if bytes.len() >= SnapshotHeader::LENGTH {
                    info!("Downloaded snapshot header from {}.", download_url);

                    bytes.resize(SnapshotHeader::LENGTH, 0);
                    let mut slice: &[u8] = &bytes;

                    return Ok(Some((SnapshotHeader::unpack(&mut slice)?, download_url)));
                }
            }
        }
        Err(e) => warn!("Downloading snapshot header failed: {:?}.", e.to_string()),
    }

    Ok(None)
}

async fn download_full_snapshot(path: &Path, download_url: &str) -> Result<(), Error> {
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
