use axum::response::Response;
use image::DynamicImage;
use reqwest::StatusCode;
use std::path::PathBuf;
use tracing::info;

use crate::{
    constants::CDN_ROOT,
    env::state::AppState,
    utils::{
        fetch::fetch_and_cache,
        http::response_file,
        img::{save_image_to_webp, save_resized_image},
        path::{get_original_path, get_resize_width_from_path},
    },
};

pub async fn process_image_request(
    state: &AppState,
    path: &str,
    file_path: &PathBuf,
) -> Result<Response, StatusCode> {
    if file_path.exists() {
        info!("File exists, respond with Rust: {:?}", file_path);
        return Ok(response_file(file_path).await);
    }

    let resize_width = get_resize_width_from_path(path);
    let convert_to_webp = path.ends_with(".webp");
    let original_path = get_original_path(path, resize_width.is_some());
    let original_file_path = PathBuf::from(format!("{}/images/{}", CDN_ROOT, original_path));

    if !original_file_path.exists()
        && fetch_and_cache(state.host.clone(), &original_file_path, &original_path)
            .await
            .is_err()
    {
        return Err(StatusCode::NOT_FOUND);
    }

    if resize_width.is_none() && !convert_to_webp {
        return Ok(response_file(file_path).await);
    }

    let image = open_image(&original_file_path)?;

    if !convert_to_webp {
        return Ok(save_resized_image(image, resize_width, &original_file_path, file_path).await);
    }

    let path_with_webp = format!("{}.webp", original_path);
    let file_path_with_webp = PathBuf::from(format!("{}/images/{}", CDN_ROOT, path_with_webp));

    save_image_to_webp(&image, &file_path_with_webp)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let image_webp = open_image(&file_path_with_webp)?;

    Ok(save_resized_image(image_webp, resize_width, &file_path_with_webp, file_path).await)
}

fn open_image(file_path: &PathBuf) -> Result<DynamicImage, StatusCode> {
    image::open(file_path).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}
