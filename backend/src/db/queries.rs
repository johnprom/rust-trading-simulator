use crate::models::{UserData, UserId};
use sqlx::{SqlitePool, Row};
use std::collections::HashMap;

pub async fn get_user(pool: &SqlitePool, user_id: &UserId) -> Result<Option<UserData>, sqlx::Error> {
    let row = sqlx::query(
        r#"
        SELECT user_id, username, cash_balance, asset_balances
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

            let asset_balances: HashMap<String, f64> = serde_json::from_str(&asset_balances_str)
                .unwrap_or_default();

            Ok(Some(UserData {
                username,
                cash_balance,
                asset_balances,
            }))
        }
        None => Ok(None),
    }
}

pub async fn save_user(pool: &SqlitePool, user_id: &UserId, user: &UserData) -> Result<(), sqlx::Error> {
    let asset_balances_json = serde_json::to_string(&user.asset_balances)
        .unwrap_or_else(|_| "{}".to_string());

    sqlx::query(
        r#"
        INSERT INTO users (user_id, username, cash_balance, asset_balances)
        VALUES (?, ?, ?, ?)
        ON CONFLICT(user_id) DO UPDATE SET
            username = excluded.username,
            cash_balance = excluded.cash_balance,
            asset_balances = excluded.asset_balances
        "#
    )
    .bind(user_id)
    .bind(&user.username)
    .bind(user.cash_balance)
    .bind(asset_balances_json)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn load_all_users(pool: &SqlitePool) -> Result<HashMap<UserId, UserData>, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT user_id, username, cash_balance, asset_balances
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

        let asset_balances: HashMap<String, f64> = serde_json::from_str(&asset_balances_str)
            .unwrap_or_default();

        users.insert(
            user_id,
            UserData {
                username,
                cash_balance,
                asset_balances,
            },
        );
    }

    Ok(users)
}
