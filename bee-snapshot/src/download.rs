// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::Error;

use log::{error, info, warn};

use std::{fs::File, io::copy, path::Path};

// TODO copy is not really streaming ?
// TODO temporary file until fully downloaded ?
pub async fn download_snapshot_file(file_path: &str, download_urls: &Vec<String>) -> Result<(), Error> {
    let file_name = match file_path.rfind('/') {
        Some(index) => &file_path[index + 1..],
        None => return Err(Error::InvalidFilePath(file_path.to_owned())),
    };

    for url in download_urls {
        let url = url.to_owned() + file_name;

        info!("Downloading snapshot file {}...", url);
        match reqwest::get(&url).await {
            Ok(res) => match File::create(file_path) {
                // TODO unwrap
                Ok(mut file) => match copy(&mut res.bytes().await.unwrap().as_ref(), &mut file) {
                    Ok(_) => break,
                    Err(e) => warn!("Copying snapshot file failed: {:?}.", e),
                },
                Err(e) => warn!("Creating snapshot file failed: {:?}.", e),
            },
            Err(e) => warn!("Downloading snapshot file failed: {:?}.", e),
        }
    }

    // TODO here or outside ?
    if Path::new(file_path).exists() {
        Ok(())
    } else {
        error!("No working download source available.");
        Err(Error::NoDownloadSourceAvailable)
    }
}
