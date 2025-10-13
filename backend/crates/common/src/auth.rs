use axum::{extract::FromRequestParts, http::request::Parts};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

use crate::{
    config::AppConfig,
    error::{AppError, AppResult},
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: Uuid,
    pub exp: i64,
}

#[derive(Clone)]
pub struct JwtKeys {
    pub enc: EncodingKey,
    pub dec: DecodingKey,
}

impl JwtKeys {
    pub fn from_secret(secret: &str) -> Self {
        Self {
            enc: EncodingKey::from_secret(secret.as_bytes()),
            dec: DecodingKey::from_secret(secret.as_bytes()),
        }
    }
}

pub fn issue_jwt(user_id: Uuid, keys: &JwtKeys, ttl_minutes: i64) -> Result<String, AppError> {
    let exp = (OffsetDateTime::now_utc() + Duration::minutes(ttl_minutes)).unix_timestamp();
    let claims = Claims { sub: user_id, exp };
    let header = Header {
        alg: Algorithm::HS256,
        ..Default::default()
    };
    encode(&header, &claims, &keys.enc).map_err(|e| AppError::Other(e.into()))
}

pub fn verify_jwt(token: &str, keys: &JwtKeys) -> Result<Claims, AppError> {
    let mut val = Validation::new(Algorithm::HS256);
    val.validate_exp = true;
    decode::<Claims>(token, &keys.dec, &val)
        .map(|td| td.claims)
        .map_err(|_| AppError::Unauthorized)
}

#[derive(Clone, Debug)]
pub struct AuthUser {
    pub user_id: Uuid,
    pub claims: Claims,
}

impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync + 'static,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let auth = parts
            .headers
            .get("Authorization")
            .and_then(|h| h.to_str().ok())
            .ok_or(AppError::Unauthorized)?;

        let token = auth.strip_prefix("Bearer ").ok_or(AppError::Unauthorized)?;

        let keys = parts.extensions.get::<JwtKeys>().ok_or_else(|| {
            AppError::Other(anyhow::anyhow!("missing JwtKeys in request extensions"))
        })?;

        let claims = verify_jwt(token, keys)?;
        Ok(AuthUser {
            user_id: claims.sub,
            claims,
        })
    }
}

pub fn jwt_keys_from_config(cfg: &AppConfig) -> JwtKeys {
    JwtKeys::from_secret(&cfg.jwt_secret)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    Patient,
    Doctor,
    Admin,
}

impl Role {
    pub fn as_str(self) -> &'static str {
        match self {
            Role::Patient => "PATIENT",
            Role::Doctor => "DOCTOR",
            Role::Admin => "ADMIN",
        }
    }
}

pub async fn user_has_role(pool: &PgPool, user_id: Uuid, role: Role) -> AppResult<bool> {
    let exists = sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS(
            SELECT 1
            FROM user_roles
            WHERE user_id = $1
              AND role = $2::role_type
        )
        "#,
    )
    .bind(user_id)
    .bind(role.as_str())
    .fetch_one(pool)
    .await?;

    Ok(exists)
}

pub async fn ensure_user_role(pool: &PgPool, user_id: Uuid, role: Role) -> AppResult<()> {
    if user_has_role(pool, user_id, role).await? {
        Ok(())
    } else {
        Err(AppError::Forbidden)
    }
}
