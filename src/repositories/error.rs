use thiserror::Error;
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum TenantRepositoryError {
	#[error("Database error: {0}")]
	Database(#[from] sqlx::Error),

	#[error("Tenant not found: {0}")]
	TenantNotFound(Uuid),

	#[error("User not found: {0}")]
	UserNotFound(Uuid),

	#[error("Resource not found: {resource_type} {resource_id}")]
	ResourceNotFound {
		resource_type: String,
		resource_id: String,
	},

	#[error("Quota exceeded: {0}")]
	QuotaExceeded(String),

	#[error("Access denied: {0}")]
	AccessDenied(String),

	#[error("Invalid configuration: {0}")]
	InvalidConfiguration(String),

	#[error("Validation error: {0}")]
	ValidationError(String),

	#[error("Already exists: {resource_type} {resource_id}")]
	AlreadyExists {
		resource_type: String,
		resource_id: String,
	},

	#[error("Internal error: {0}")]
	Internal(String),
}
