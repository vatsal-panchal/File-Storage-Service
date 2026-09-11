use crate::engine::StorageEngine;
use crate::error::AppError;
use crate::models::{DeleteResponse, FileListResponse, HealthResponse, UploadResponse};
use axum::{
    body::Body,
    extract::{Multipart, Path, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use std::sync::Arc;

pub async fn health(
    State(engine): State<Arc<StorageEngine>>,
) -> Result<Json<HealthResponse>, AppError> {
    let (count, total_bytes) = engine.get_stats();
    Ok(Json(HealthResponse {
        status: "ok".to_string(),
        files_count: count,
        total_bytes,
    }))
}

pub async fn upload_file(
    State(engine): State<Arc<StorageEngine>>,
    mut multipart: Multipart,
) -> Result<(StatusCode, Json<UploadResponse>), AppError> {
    let mut file_data: Option<Vec<u8>> = None;
    let mut filename: Option<String> = None;
    let mut content_type: Option<String> = None;

    while let Some(field) = multipart.next_field().await? {
        let field_name = field.name().unwrap_or_default().to_string();
        if field_name == "file" || field.file_name().is_some() {
            if let Some(name) = field.file_name() {
                filename = Some(name.to_string());
            }
            if let Some(ct) = field.content_type() {
                content_type = Some(ct.to_string());
            }
            let bytes = field.bytes().await?;
            file_data = Some(bytes.to_vec());
            break;
        }
    }

    let data = file_data.ok_or_else(|| {
        AppError::BadRequest("No file field provided in multipart form".to_string())
    })?;

    let name = filename.unwrap_or_else(|| "uploaded_file.bin".to_string());
    let metadata = engine.save_file(name, content_type, &data)?;

    Ok((
        StatusCode::CREATED,
        Json(UploadResponse {
            success: true,
            file: metadata,
        }),
    ))
}

pub async fn list_files(
    State(engine): State<Arc<StorageEngine>>,
) -> Result<Json<FileListResponse>, AppError> {
    let files = engine.list_files();
    Ok(Json(FileListResponse {
        success: true,
        count: files.len(),
        files,
    }))
}

pub async fn get_file_info(
    State(engine): State<Arc<StorageEngine>>,
    Path(id): Path<String>,
) -> Result<Json<crate::models::FileMetadata>, AppError> {
    let meta = engine.get_metadata(&id)?;
    Ok(Json(meta))
}

pub async fn download_file(
    State(engine): State<Arc<StorageEngine>>,
    Path(id): Path<String>,
) -> Result<Response, AppError> {
    let (meta, bytes) = engine.get_file(&id)?;

    let mut headers = HeaderMap::new();
    let content_disposition = format!("attachment; filename=\"{}\"", meta.filename);
    if let Ok(val) = HeaderValue::from_str(&content_disposition) {
        headers.insert(header::CONTENT_DISPOSITION, val);
    }
    if let Ok(val) = HeaderValue::from_str(&meta.content_type) {
        headers.insert(header::CONTENT_TYPE, val);
    }

    Ok((StatusCode::OK, headers, Body::from(bytes)).into_response())
}

pub async fn delete_file(
    State(engine): State<Arc<StorageEngine>>,
    Path(id): Path<String>,
) -> Result<Json<DeleteResponse>, AppError> {
    engine.delete_file(&id)?;
    Ok(Json(DeleteResponse {
        success: true,
        message: format!("File '{}' deleted successfully", id),
    }))
}
