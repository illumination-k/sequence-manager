pub mod cache;

use std::path::Path;

use anyhow::Result;

#[async_trait::async_trait]
pub trait Downloader {
    async fn get_file_url(&mut self, taxonomy_id: Option<u32>) -> Result<()>;
    async fn download_and_save<S: Into<String>, P: AsRef<Path>>(&mut self, url: S, path: P) -> Result<()>;
}