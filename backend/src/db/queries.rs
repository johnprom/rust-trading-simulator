use crate::models::{UserData, UserId};
use crate::services::auth_service::{self, AuthError};
use sqlx::{SqlitePool, Row};
use std::collections::HashMap;

pub async fn get_user(pool: &SqlitePool, user_id: &UserId) -> Result<Option<UserData>, sqlx::Error> {
    let row = sqlx::query(
        r#"
        SELECT user_id, username, cash_balance, asset_balances, trade_history
        FROM users
        WHERE user_id = ?
        "#
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    match row {
        Some(r) => {
            let username: String = r.get("username");
            let cash_balance: f64 = r.get("cash_balance");
            let asset_balances_str: String = r.get("asset_balances");
            let trade_history_str: String = r.get("trade_history");

            let mut asset_balances: HashMap<String, f64> = serde_json::from_str(&asset_balances_str)
                .unwrap_or_default();
            let trade_history = serde_json::from_str(&trade_history_str)
                .unwrap_or_default();

            // Migration: Move cash_balance to USD asset if not already there
            if !asset_balances.contains_key("USD") && cash_balance > 0.0 {
                asset_balances.insert("USD".to_string(), cash_balance);
            }

            Ok(Some(UserData {
                username,
                cash_balance,  // Keep for backward compat
                asset_balances,
                trade_history,
            }))
        }
        None => Ok(None),
    }
}

pub async fn save_user(pool: &SqlitePool, user_id: &UserId, user: &UserData) -> Result<(), sqlx::Error> {
    let asset_balances_json = serde_json::to_string(&user.asset_balances)
        .unwrap_or_else(|_| "{}".to_string());
    let trade_history_json = serde_json::to_string(&user.trade_history)
        .unwrap_or_else(|_| "[]".to_string());

    sqlx::query(
        r#"
        INSERT INTO users (user_id, username, cash_balance, asset_balances, trade_history)
        VALUES (?, ?, ?, ?, ?)
        ON CONFLICT(user_id) DO UPDATE SET
            username = excluded.username,
            cash_balance = excluded.cash_balance,
            asset_balances = excluded.asset_balances,
            trade_history = excluded.trade_history
        "#
    )
    .bind(user_id)
    .bind(&user.username)
    .bind(user.cash_balance)
    .bind(asset_balances_json)
    .bind(trade_history_json)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn load_all_users(pool: &SqlitePool) -> Result<HashMap<UserId, UserData>, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT user_id, username, cash_balance, asset_balances, trade_history
        FROM users
        "#
    )
    .fetch_all(pool)
    .await?;

    let mut users = HashMap::new();
    for row in rows {
        let user_id: String = row.get("user_id");
        let username: String = row.get("username");
        let cash_balance: f64 = row.get("cash_balance");
        let asset_balances_str: String = row.get("asset_balances");
        let trade_history_str: String = row.get("trade_history");

        let mut asset_balances: HashMap<String, f64> = serde_json::from_str(&asset_balances_str)
            .unwrap_or_default();
        let trade_history = serde_json::from_str(&trade_history_str)
            .unwrap_or_default();

        // Migration: Move cash_balance to USD asset if not already there
        if !asset_balances.contains_key("USD") && cash_balance > 0.0 {
            asset_balances.insert("USD".to_string(), cash_balance);
        }

        users.insert(
            user_id,
            UserData {
                username,
                cash_balance,  // Keep for backward compat
                asset_balances,
                trade_history,
            },
        );
    }

    Ok(users)
}

pub async fn delete_user(pool: &SqlitePool, user_id: &UserId) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        DELETE FROM users WHERE user_id = ?
        "#
    )
    .bind(user_id)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn create_user(
    pool: &SqlitePool,
    user_id: &UserId,
    username: &str,
    password: &str,
) -> Result<(), AuthError> {
    // Check if username already exists
    let existing = sqlx::query(
        r#"
        SELECT user_id FROM users WHERE username = ?
        "#
    )
    .bind(username)
    .fetch_optional(pool)
    .await
    .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

    if existing.is_some() {
        return Err(AuthError::UserAlreadyExists);
    }

    // Hash password
    let password_hash = auth_service::hash_password(password)?;

    // Create user data
    let user_data = UserData::new(username.to_string());
    let asset_balances_json = serde_json::to_string(&user_data.asset_balances)
        .unwrap_or_else(|_| "{}".to_string());
    let trade_history_json = serde_json::to_string(&user_data.trade_history)
        .unwrap_or_else(|_| "[]".to_string());

    // Insert user with password
    sqlx::query(
        r#"
        INSERT INTO users (user_id, username, cash_balance, asset_balances, trade_history, password_hash)
        VALUES (?, ?, ?, ?, ?, ?)
        "#
    )
    .bind(user_id)
    .bind(username)
    .bind(user_data.cash_balance)
    .bind(asset_balances_json)
    .bind(trade_history_json)
    .bind(password_hash)
    .execute(pool)
    .await
    .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

    Ok(())
}

pub async fn get_user_by_username(
    pool: &SqlitePool,
    username: &str,
) -> Result<Option<(UserId, String)>, AuthError> {
    let row = sqlx::query(
        r#"
        SELECT user_id, password_hash FROM users WHERE username = ?
        "#
    )
    .bind(username)
    .fetch_optional(pool)
    .await
    .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

    match row {
        Some(r) => {
            let user_id: String = r.get("user_id");
            let password_hash: Option<String> = r.get("password_hash");

            match password_hash {
                Some(hash) => Ok(Some((user_id, hash))),
                None => Err(AuthError::InvalidCredentials),
            }
        }
        None => Ok(None),
    }
}

pub async fn verify_user_credentials(
    pool: &SqlitePool,
    username: &str,
    password: &str,
) -> Result<UserId, AuthError> {
    match get_user_by_username(pool, username).await? {
        Some((user_id, password_hash)) => {
            if auth_service::verify_password(password, &password_hash)? {
                Ok(user_id)
            } else {
                Err(AuthError::InvalidCredentials)
            }
        }
        None => Err(AuthError::InvalidCredentials),
    }
}
