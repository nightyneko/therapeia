use axum::{extract::FromRequestParts, http::request::Parts};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

use crate::{config::AppConfig, error::AppError};

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
    let mut header = Header::default();
    header.alg = Algorithm::HS256;
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
