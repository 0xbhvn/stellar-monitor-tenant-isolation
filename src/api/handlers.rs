use axum::{
	extract::{Path, Query, State},
	http::StatusCode,
	response::IntoResponse,
	Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::*;
use crate::repositories::*;
use crate::services::*;

#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
	pub limit: Option<i64>,
	pub offset: Option<i64>,
}

impl Default for PaginationQuery {
	fn default() -> Self {
		Self {
			limit: Some(20),
			offset: Some(0),
		}
	}
}

#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
	pub data: T,
	pub meta: Option<MetaData>,
}

#[derive(Debug, Serialize)]
pub struct MetaData {
	pub total: Option<i64>,
	pub limit: i64,
	pub offset: i64,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
	pub error: String,
	pub code: String,
}

// Monitor handlers
pub async fn create_monitor<M, N, T, TR, A>(
	State(state): State<super::routes::AppState<M, N, T, TR, A>>,
	Json(request): Json<CreateMonitorRequest>,
) -> Result<impl IntoResponse, ApiError>
where
	M: MonitorServiceTrait,
	N: NetworkServiceTrait,
	T: TriggerServiceTrait,
	TR: TenantRepositoryTrait,
	A: AuditServiceTrait,
{
	let monitor = state.monitor_service.create_monitor(request).await?;
	Ok((
		StatusCode::CREATED,
		Json(ApiResponse {
			data: monitor,
			meta: None,
		}),
	))
}

pub async fn get_monitor<M, N, T, TR, A>(
	State(state): State<super::routes::AppState<M, N, T, TR, A>>,
	Path(monitor_id): Path<String>,
) -> Result<impl IntoResponse, ApiError>
where
	M: MonitorServiceTrait,
	N: NetworkServiceTrait,
	T: TriggerServiceTrait,
	TR: TenantRepositoryTrait,
	A: AuditServiceTrait,
{
	let monitor = state.monitor_service.get_monitor(&monitor_id).await?;
	Ok(Json(ApiResponse {
		data: monitor,
		meta: None,
	}))
}

pub async fn update_monitor<M, N, T, TR, A>(
	State(state): State<super::routes::AppState<M, N, T, TR, A>>,
	Path(monitor_id): Path<String>,
	Json(request): Json<UpdateMonitorRequest>,
) -> Result<impl IntoResponse, ApiError>
where
	M: MonitorServiceTrait,
	N: NetworkServiceTrait,
	T: TriggerServiceTrait,
	TR: TenantRepositoryTrait,
	A: AuditServiceTrait,
{
	let monitor = state
		.monitor_service
		.update_monitor(&monitor_id, request)
		.await?;
	Ok(Json(ApiResponse {
		data: monitor,
		meta: None,
	}))
}

pub async fn delete_monitor<M, N, T, TR, A>(
	State(state): State<super::routes::AppState<M, N, T, TR, A>>,
	Path(monitor_id): Path<String>,
) -> Result<impl IntoResponse, ApiError>
where
	M: MonitorServiceTrait,
	N: NetworkServiceTrait,
	T: TriggerServiceTrait,
	TR: TenantRepositoryTrait,
	A: AuditServiceTrait,
{
	state.monitor_service.delete_monitor(&monitor_id).await?;
	Ok(StatusCode::NO_CONTENT)
}

pub async fn list_monitors<M, N, T, TR, A>(
	State(state): State<super::routes::AppState<M, N, T, TR, A>>,
	Query(pagination): Query<PaginationQuery>,
) -> Result<impl IntoResponse, ApiError>
where
	M: MonitorServiceTrait,
	N: NetworkServiceTrait,
	T: TriggerServiceTrait,
	TR: TenantRepositoryTrait,
	A: AuditServiceTrait,
{
	let limit = pagination.limit.unwrap_or(20);
	let offset = pagination.offset.unwrap_or(0);

	let monitors = state.monitor_service.list_monitors(limit, offset).await?;
	let total = state.monitor_service.get_monitor_count().await?;

	Ok(Json(ApiResponse {
		data: monitors,
		meta: Some(MetaData {
			total: Some(total),
			limit,
			offset,
		}),
	}))
}

// Network handlers
pub async fn create_network<M, N, T, TR, A>(
	State(state): State<super::routes::AppState<M, N, T, TR, A>>,
	Json(request): Json<CreateNetworkRequest>,
) -> Result<impl IntoResponse, ApiError>
where
	M: MonitorServiceTrait,
	N: NetworkServiceTrait,
	T: TriggerServiceTrait,
	TR: TenantRepositoryTrait,
	A: AuditServiceTrait,
{
	let network = state.network_service.create_network(request).await?;
	Ok((
		StatusCode::CREATED,
		Json(ApiResponse {
			data: network,
			meta: None,
		}),
	))
}

pub async fn get_network<M, N, T, TR, A>(
	State(state): State<super::routes::AppState<M, N, T, TR, A>>,
	Path(network_id): Path<String>,
) -> Result<impl IntoResponse, ApiError>
where
	M: MonitorServiceTrait,
	N: NetworkServiceTrait,
	T: TriggerServiceTrait,
	TR: TenantRepositoryTrait,
	A: AuditServiceTrait,
{
	let network = state.network_service.get_network(&network_id).await?;
	Ok(Json(ApiResponse {
		data: network,
		meta: None,
	}))
}

pub async fn update_network<M, N, T, TR, A>(
	State(state): State<super::routes::AppState<M, N, T, TR, A>>,
	Path(network_id): Path<String>,
	Json(request): Json<UpdateNetworkRequest>,
) -> Result<impl IntoResponse, ApiError>
where
	M: MonitorServiceTrait,
	N: NetworkServiceTrait,
	T: TriggerServiceTrait,
	TR: TenantRepositoryTrait,
	A: AuditServiceTrait,
{
	let network = state
		.network_service
		.update_network(&network_id, request)
		.await?;
	Ok(Json(ApiResponse {
		data: network,
		meta: None,
	}))
}

pub async fn delete_network<M, N, T, TR, A>(
	State(state): State<super::routes::AppState<M, N, T, TR, A>>,
	Path(network_id): Path<String>,
) -> Result<impl IntoResponse, ApiError>
where
	M: MonitorServiceTrait,
	N: NetworkServiceTrait,
	T: TriggerServiceTrait,
	TR: TenantRepositoryTrait,
	A: AuditServiceTrait,
{
	state.network_service.delete_network(&network_id).await?;
	Ok(StatusCode::NO_CONTENT)
}

pub async fn list_networks<M, N, T, TR, A>(
	State(state): State<super::routes::AppState<M, N, T, TR, A>>,
	Query(pagination): Query<PaginationQuery>,
) -> Result<impl IntoResponse, ApiError>
where
	M: MonitorServiceTrait,
	N: NetworkServiceTrait,
	T: TriggerServiceTrait,
	TR: TenantRepositoryTrait,
	A: AuditServiceTrait,
{
	let limit = pagination.limit.unwrap_or(20);
	let offset = pagination.offset.unwrap_or(0);

	let networks = state.network_service.list_networks(limit, offset).await?;
	let total = state.network_service.get_network_count().await?;

	Ok(Json(ApiResponse {
		data: networks,
		meta: Some(MetaData {
			total: Some(total),
			limit,
			offset,
		}),
	}))
}

// Trigger handlers
pub async fn create_trigger<M, N, T, TR, A>(
	State(state): State<super::routes::AppState<M, N, T, TR, A>>,
	Json(request): Json<CreateTriggerRequest>,
) -> Result<impl IntoResponse, ApiError>
where
	M: MonitorServiceTrait,
	N: NetworkServiceTrait,
	T: TriggerServiceTrait,
	TR: TenantRepositoryTrait,
	A: AuditServiceTrait,
{
	let trigger = state.trigger_service.create_trigger(request).await?;
	Ok((
		StatusCode::CREATED,
		Json(ApiResponse {
			data: trigger,
			meta: None,
		}),
	))
}

pub async fn get_trigger<M, N, T, TR, A>(
	State(state): State<super::routes::AppState<M, N, T, TR, A>>,
	Path(trigger_id): Path<String>,
) -> Result<impl IntoResponse, ApiError>
where
	M: MonitorServiceTrait,
	N: NetworkServiceTrait,
	T: TriggerServiceTrait,
	TR: TenantRepositoryTrait,
	A: AuditServiceTrait,
{
	let trigger = state.trigger_service.get_trigger(&trigger_id).await?;
	Ok(Json(ApiResponse {
		data: trigger,
		meta: None,
	}))
}

pub async fn update_trigger<M, N, T, TR, A>(
	State(state): State<super::routes::AppState<M, N, T, TR, A>>,
	Path(trigger_id): Path<String>,
	Json(request): Json<UpdateTriggerRequest>,
) -> Result<impl IntoResponse, ApiError>
where
	M: MonitorServiceTrait,
	N: NetworkServiceTrait,
	T: TriggerServiceTrait,
	TR: TenantRepositoryTrait,
	A: AuditServiceTrait,
{
	let trigger = state
		.trigger_service
		.update_trigger(&trigger_id, request)
		.await?;
	Ok(Json(ApiResponse {
		data: trigger,
		meta: None,
	}))
}

pub async fn delete_trigger<M, N, T, TR, A>(
	State(state): State<super::routes::AppState<M, N, T, TR, A>>,
	Path(trigger_id): Path<String>,
) -> Result<impl IntoResponse, ApiError>
where
	M: MonitorServiceTrait,
	N: NetworkServiceTrait,
	T: TriggerServiceTrait,
	TR: TenantRepositoryTrait,
	A: AuditServiceTrait,
{
	state.trigger_service.delete_trigger(&trigger_id).await?;
	Ok(StatusCode::NO_CONTENT)
}

pub async fn list_triggers<M, N, T, TR, A>(
	State(state): State<super::routes::AppState<M, N, T, TR, A>>,
	Query(pagination): Query<PaginationQuery>,
) -> Result<impl IntoResponse, ApiError>
where
	M: MonitorServiceTrait,
	N: NetworkServiceTrait,
	T: TriggerServiceTrait,
	TR: TenantRepositoryTrait,
	A: AuditServiceTrait,
{
	let limit = pagination.limit.unwrap_or(20);
	let offset = pagination.offset.unwrap_or(0);

	let triggers = state.trigger_service.list_triggers(limit, offset).await?;

	Ok(Json(ApiResponse {
		data: triggers,
		meta: Some(MetaData {
			total: None, // TODO: Add count method
			limit,
			offset,
		}),
	}))
}

pub async fn list_triggers_by_monitor<M, N, T, TR, A>(
	State(state): State<super::routes::AppState<M, N, T, TR, A>>,
	Path(monitor_id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError>
where
	M: MonitorServiceTrait,
	N: NetworkServiceTrait,
	T: TriggerServiceTrait,
	TR: TenantRepositoryTrait,
	A: AuditServiceTrait,
{
	let triggers = state
		.trigger_service
		.list_triggers_by_monitor(monitor_id)
		.await?;
	Ok(Json(ApiResponse {
		data: triggers,
		meta: None,
	}))
}

// Health check
pub async fn health_check() -> impl IntoResponse {
	Json(serde_json::json!({
		"status": "healthy",
		"timestamp": chrono::Utc::now(),
	}))
}

// Error handling
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
	#[error("Service error: {0}")]
	Service(#[from] ServiceError),

	#[error("Bad request: {0}")]
	BadRequest(String),

	#[error("Unauthorized")]
	Unauthorized,

	#[error("Not found")]
	NotFound,

	#[error("Internal server error")]
	Internal,
}

impl IntoResponse for ApiError {
	fn into_response(self) -> axum::response::Response {
		let (status, code, message) = match self {
			ApiError::Service(ref err) => match err {
				ServiceError::AccessDenied(_) => {
					(StatusCode::FORBIDDEN, "ACCESS_DENIED", err.to_string())
				}
				ServiceError::QuotaExceeded(_) => (
					StatusCode::TOO_MANY_REQUESTS,
					"QUOTA_EXCEEDED",
					err.to_string(),
				),
				ServiceError::ValidationError(_) => {
					(StatusCode::BAD_REQUEST, "VALIDATION_ERROR", err.to_string())
				}
				ServiceError::Repository(ref repo_err) => match repo_err {
					crate::repositories::TenantRepositoryError::ResourceNotFound { .. } => (
						StatusCode::NOT_FOUND,
						"NOT_FOUND",
						"Resource not found".to_string(),
					),
					crate::repositories::TenantRepositoryError::AlreadyExists { .. } => (
						StatusCode::CONFLICT,
						"ALREADY_EXISTS",
						"Resource already exists".to_string(),
					),
					_ => (
						StatusCode::INTERNAL_SERVER_ERROR,
						"INTERNAL_ERROR",
						"Internal server error".to_string(),
					),
				},
				_ => (
					StatusCode::INTERNAL_SERVER_ERROR,
					"INTERNAL_ERROR",
					"Internal server error".to_string(),
				),
			},
			ApiError::BadRequest(_) => (StatusCode::BAD_REQUEST, "BAD_REQUEST", self.to_string()),
			ApiError::Unauthorized => (
				StatusCode::UNAUTHORIZED,
				"UNAUTHORIZED",
				"Unauthorized".to_string(),
			),
			ApiError::NotFound => (StatusCode::NOT_FOUND, "NOT_FOUND", "Not found".to_string()),
			ApiError::Internal => (
				StatusCode::INTERNAL_SERVER_ERROR,
				"INTERNAL_ERROR",
				"Internal server error".to_string(),
			),
		};

		let body = Json(ErrorResponse {
			error: message,
			code: code.to_string(),
		});

		(status, body).into_response()
	}
}
