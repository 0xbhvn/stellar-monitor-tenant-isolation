use argon2::password_hash::{rand_core::OsRng, SaltString};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::User;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
	pub sub: Uuid, // user_id
	pub email: String,
	pub exp: i64, // expiration timestamp
	pub iat: i64, // issued at timestamp
}

#[derive(Clone)]
pub struct AuthService {
	jwt_secret: String,
}

impl AuthService {
	pub fn new(jwt_secret: String) -> Self {
		Self { jwt_secret }
	}

	pub fn hash_password(&self, password: &str) -> Result<String, argon2::password_hash::Error> {
		let salt = SaltString::generate(&mut OsRng);
		let argon2 = Argon2::default();
		let password_hash = argon2.hash_password(password.as_bytes(), &salt)?;
		Ok(password_hash.to_string())
	}

	pub fn verify_password(
		&self,
		password: &str,
		hash: &str,
	) -> Result<bool, argon2::password_hash::Error> {
		let parsed_hash = PasswordHash::new(hash)?;
		let argon2 = Argon2::default();
		match argon2.verify_password(password.as_bytes(), &parsed_hash) {
			Ok(()) => Ok(true),
			Err(argon2::password_hash::Error::Password) => Ok(false),
			Err(e) => Err(e),
		}
	}

	pub fn generate_jwt(&self, user: &User) -> Result<String, jsonwebtoken::errors::Error> {
		self.generate_jwt_with_expiry(user, Duration::hours(1)) // 1 hour for access token
	}

	pub fn generate_refresh_token(
		&self,
		user: &User,
	) -> Result<String, jsonwebtoken::errors::Error> {
		self.generate_jwt_with_expiry(user, Duration::days(30)) // 30 days for refresh token
	}

	fn generate_jwt_with_expiry(
		&self,
		user: &User,
		expiry_duration: Duration,
	) -> Result<String, jsonwebtoken::errors::Error> {
		let now = Utc::now();
		let expiration = now + expiry_duration;

		let claims = Claims {
			sub: user.id,
			email: user.email.clone(),
			exp: expiration.timestamp(),
			iat: now.timestamp(),
		};

		encode(
			&Header::default(),
			&claims,
			&EncodingKey::from_secret(self.jwt_secret.as_bytes()),
		)
	}

	pub fn verify_jwt(&self, token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
		decode::<Claims>(
			token,
			&DecodingKey::from_secret(self.jwt_secret.as_bytes()),
			&Validation::default(),
		)
		.map(|data| data.claims)
	}

	pub fn generate_api_key(&self) -> String {
		// Generate a secure random API key
		use base64::{engine::general_purpose, Engine as _};
		use rand::Rng;
		let mut rng = rand::thread_rng();
		let key: [u8; 32] = rng.gen();
		general_purpose::STANDARD.encode(key)
	}
}
