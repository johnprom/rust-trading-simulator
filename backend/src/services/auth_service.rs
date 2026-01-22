use bcrypt::{hash, verify, DEFAULT_COST};
use uuid::Uuid;

#[derive(Debug)]
pub enum AuthError {
    InvalidCredentials,
    UserAlreadyExists,
    HashError(String),
    DatabaseError(String),
}

impl std::fmt::Display for AuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuthError::InvalidCredentials => write!(f, "Invalid username or password"),
            AuthError::UserAlreadyExists => write!(f, "Username already exists"),
            AuthError::HashError(msg) => write!(f, "Password hashing error: {}", msg),
            AuthError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
        }
    }
}

impl std::error::Error for AuthError {}

pub fn hash_password(password: &str) -> Result<String, AuthError> {
    hash(password, DEFAULT_COST)
        .map_err(|e| AuthError::HashError(e.to_string()))
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool, AuthError> {
    verify(password, hash)
        .map_err(|e| AuthError::HashError(e.to_string()))
}

pub fn generate_user_id() -> String {
    Uuid::new_v4().to_string()
}
