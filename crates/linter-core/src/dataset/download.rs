use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use thiserror::Error;
use tracing::info;

pub const DEFAULT_DATASET_URL: &str =
    "https://huggingface.co/datasets/FronkonGames/steam-games-dataset/resolve/main/data/train-00000-of-00001.parquet";

#[derive(Debug, Error)]
pub enum DownloadError {
    #[error("HTTP request failed: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("Filesystem error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Download interrupted or server returned error status: {0}")]
    HttpStatus(reqwest::StatusCode),
}

pub struct DatasetDownloader;

impl DatasetDownloader {
    pub async fn download(
        url: Option<&str>,
        target_path: impl AsRef<Path>,
        force: bool,
    ) -> Result<PathBuf, DownloadError> {
        let download_url = url.unwrap_or(DEFAULT_DATASET_URL);
        let path = target_path.as_ref().to_path_buf();

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Check if file already exists and is non-empty
        if path.exists() && !force {
            let metadata = fs::metadata(&path)?;
            if metadata.len() > 100_000_000 {
                info!(
                    "Dataset parquet file already exists at {} ({:.1} MB). Skipping download.",
                    path.display(),
                    metadata.len() as f64 / 1_000_000.0
                );
                return Ok(path);
            }
        }

        info!("Connecting to Hugging Face dataset: {}", download_url);
        let client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::limited(10))
            .build()?;

        let mut res = client.get(download_url).send().await?;
        if !res.status().is_success() {
            return Err(DownloadError::HttpStatus(res.status()));
        }

        let total_size = res.content_length().unwrap_or(0);
        info!(
            "Downloading dataset ({:.1} MB)...",
            total_size as f64 / 1_000_000.0
        );

        let mut file = File::create(&path)?;
        let mut downloaded: u64 = 0;

        while let Some(chunk) = res.chunk().await? {
            file.write_all(&chunk)?;
            downloaded += chunk.len() as u64;
        }

        file.flush()?;
        info!(
            "Successfully downloaded dataset to {} ({} bytes)",
            path.display(),
            downloaded
        );

        Ok(path)
    }
}
