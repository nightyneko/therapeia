use crate::error::AppError;
use bcrypt::{BcryptError, DEFAULT_COST, hash, verify};

/// Hash a plaintext password using bcrypt.
pub fn hash_password(plaintext: &str) -> Result<String, AppError> {
    hash(plaintext, DEFAULT_COST).map_err(|e| AppError::Other(e.into()))
}

/// Verify a plaintext password against a bcrypt hash.
pub fn verify_password(plaintext: &str, hashed: &str) -> Result<bool, AppError> {
    match verify(plaintext, hashed) {
        Ok(result) => Ok(result),
        Err(BcryptError::InvalidHash(_)) | Err(BcryptError::InvalidCost(_)) => {
            Ok(plaintext == hashed)
        }
        Err(err) => Err(AppError::Other(err.into())),
    }
}
