use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(sqlx::FromRow, Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub title: String,
    pub user_id: i32,
    pub description: Option<String>,
    pub is_completed: Option<bool>,
    pub due_date: Option<NaiveDateTime>,
}

#[derive(sqlx::FromRow, Debug, Clone, Serialize, Deserialize)]
pub struct GetTask {
    pub id: i32,
    pub title: String,
    pub description: Option<String>,
    pub user_id: Option<i32>,
    pub is_completed: Option<bool>,
    pub due_date: Option<NaiveDateTime>,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

#[derive(Debug, Deserialize)]
pub struct TaskPayload {
    pub id: Option<i32>, // Only required for update
    pub title: Option<String>,
    pub user_id: i32,
    pub description: Option<String>,
    pub is_completed: Option<bool>,
    pub due_date: Option<NaiveDateTime>,
}
