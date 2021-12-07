// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    types::snapshot::SnapshotHeader,
    workers::snapshot::{config::DownloadUrls, error::Error},
};

use bee_common::packable::Packable;
use bee_message::milestone::MilestoneIndex;

use bytes::Buf;
use futures::{future::join_all, StreamExt};
use log::{debug, info, warn};
use reqwest::Response;

use std::{io::Read, path::Path};

async fn download_snapshot_header(download_url: &str) -> Result<SnapshotHeader, Error> {
    debug!("Downloading snapshot header {}...", download_url);

    match reqwest::get(download_url).await.and_then(Response::error_for_status) {
        Ok(res) => {
            let mut stream = res.bytes_stream();
            let mut bytes = Vec::<u8>::with_capacity(SnapshotHeader::LENGTH);

            while let Some(chunk) = stream.next().await {
                let mut chunk_reader = chunk.map_err(|_| Error::DownloadingFailed)?.reader();

                let mut buf = Vec::new();
                chunk_reader.read_to_end(&mut buf)?;
                bytes.extend_from_slice(&buf);

                if bytes.len() >= SnapshotHeader::LENGTH {
                    debug!("Downloaded snapshot header from {}.", download_url);

                    let mut slice: &[u8] = &bytes[..SnapshotHeader::LENGTH];

                    return Ok(SnapshotHeader::unpack(&mut slice)?);
                }
            }
        }
        Err(e) => warn!("Downloading snapshot header failed: {:?}.", e.to_string()),
    }

    Err(Error::DownloadingFailed)
}

struct SourceInformation<'a> {
    urls: &'a DownloadUrls,
    full_header: SnapshotHeader,
    delta_header: Option<SnapshotHeader>,
}

impl<'a> SourceInformation<'a> {
    async fn download_snapshots(
        &self,
        full_snapshot_path: &Path,
        delta_snapshot_path: Option<&Path>,
    ) -> Result<(), Error> {
        download_snapshot_file(full_snapshot_path, self.urls.full()).await?;

        if let Some(delta_path) = delta_snapshot_path {
            download_snapshot_file(delta_path, self.urls.delta()).await?;
        }

        Ok(())
    }

    fn index(&self) -> MilestoneIndex {
        self.delta_header
            .as_ref()
            .map_or(self.full_header.sep_index(), SnapshotHeader::sep_index)
    }

    fn is_consistent(&self, wanted_network_id: u64) -> bool {
        if self.full_header.network_id() != wanted_network_id {
            warn!(
                "Full snapshot network ID does not match ({} != {}): {}.",
                self.full_header.network_id(),
                wanted_network_id,
                self.urls.full()
            );
            return false;
        };

        if let Some(delta_header) = self.delta_header.as_ref() {
            if delta_header.network_id() != wanted_network_id {
                warn!(
                    "Delta snapshot network ID does not match ({} != {}): {}.",
                    delta_header.network_id(),
                    wanted_network_id,
                    self.urls.delta()
                );
                return false;
            };

            if self.full_header.sep_index() > delta_header.sep_index() {
                warn!(
                    "Full snapshot SEP index is bigger than delta snapshot SEP index ({} > {}): {}.",
                    self.full_header.sep_index(),
                    delta_header.sep_index(),
                    self.urls.full()
                );
                return false;
            }

            if self.full_header.sep_index() != delta_header.ledger_index() {
                warn!(
                    "Full snapshot SEP index does not match the delta snapshot ledger index ({} != {}): {}.",
                    self.full_header.sep_index(),
                    delta_header.ledger_index(),
                    self.urls.full()
                );
                return false;
            }
        }

        true
    }
}

async fn gather_source_information(
    download_delta: bool,
    urls: &'_ DownloadUrls,
) -> Result<SourceInformation<'_>, Error> {
    let full_header = download_snapshot_header(urls.full()).await?;
    let delta_header = if download_delta {
        Some(download_snapshot_header(urls.delta()).await?)
    } else {
        None
    };

    Ok(SourceInformation {
        urls,
        full_header,
        delta_header,
    })
}

async fn download_snapshot_file(path: &Path, download_url: &str) -> Result<(), Error> {
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

    Ok(())
}

/// Tries to download the latest snapshot files from the sources specified in the `SnapshotConfig`.
/// 
/// * `wanted_network_id` - The id of the current network (typically the hash of the network name).
/// * `full_snapshot_path` - The location where the full snapshot will be stored.
/// * `full_snapshot_path` - The location where the delta snapshot will be stored.
/// * `download_urls` - The list of snapshot sources.
pub(crate) async fn download_latest_snapshot_files(
    wanted_network_id: u64,
    full_snapshot_path: &Path,
    delta_snapshot_path: Option<&Path>,
    download_urls: &[DownloadUrls],
) -> Result<(), Error> {
    let download_delta = delta_snapshot_path.is_some();

    let all_sources = join_all(
        download_urls
            .iter()
            .map(|source| gather_source_information(download_delta, source)),
    )
    .await;

    let mut available_sources = all_sources
        .into_iter()
        .flatten()
        .filter(|source| source.is_consistent(wanted_network_id))
        .collect::<Vec<SourceInformation>>();

    // Sort all available sources so that the freshest is at the end.
    available_sources.sort_by_key(SourceInformation::index);

    while let Some(source) = available_sources.pop() {
        if source
            .download_snapshots(full_snapshot_path, delta_snapshot_path)
            .await
            .is_ok()
        {
            return Ok(());
        }
    }

    Err(Error::NoDownloadSourceAvailable)
}

