use anyhow::Result;
use gotcha::{debug_handler, Json, Path, State};
use crate::error::AppError;
use crate::service::{CreateServiceRequest, Service, ServiceManager};
use crate::{AppState};

pub async fn list_services(app: State<AppState>) -> Result<Json<Vec<Service>>, AppError> {
    let service_manager = app.service_manager.lock().await;
    let services = service_manager.list_services().await?;
    Ok(Json(services))
}

pub async fn create_service(app: State<AppState>, payload: Json<CreateServiceRequest>) -> Result<Json<Service>, AppError> {
    let mut service_manager = app.service_manager.lock().await;
    let service = service_manager.create_service(payload.0).await?;
    Ok(Json(service))
}

pub async fn get_service(app: State<AppState>, paths: Path<(String,)>) -> Result<Json<Service>, AppError> {
    let service_manager = app.service_manager.lock().await;
    let service = service_manager.get_service(&paths.0.0).await?;
    Ok(Json(service))
}

pub async fn update_service(app: State<AppState>, paths: Path<(String,)> , payload: Json<CreateServiceRequest>) -> Result<Json<Service>, AppError> {
    let mut service_manager = app.service_manager.lock().await;
    let service = service_manager.update_service(&paths.0.0, payload.0).await?;
    Ok(Json(service))
}
#[debug_handler]
pub async fn delete_service(app: State<AppState>, paths: Path<(String,)>) -> Result<Json<String>, AppError> {
    let mut service_manager = app.service_manager.lock().await;
    service_manager.delete_service(&paths.0.0).await?;
    Ok(Json("Service deleted successfully".to_string()))
} 