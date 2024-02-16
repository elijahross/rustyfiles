use reqwest::Client;
use std::{fs, path::PathBuf};
use tracing::error;

pub async fn fetch_and_cache(
    host: String,
    file_path: &PathBuf,
    path: &str,
) -> Result<(), reqwest::Error> {
    let url = format!("{}{}", host, path);
    let response = match Client::new().get(&url).send().await?.error_for_status() {
        Ok(response) => response.bytes().await?,
        Err(err) => {
            error!("Failed to fetch {}", url);
            return Err(err);
        }
    };

    if let Some(parent) = file_path.parent() {
        fs::create_dir_all(parent).ok();
    }

    fs::write(file_path, &response).ok();
    Ok(())
}
