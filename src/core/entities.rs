use chrono::{DateTime, Local};

pub struct User {
    pub id: String,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub password_salt: Option<String>,
    pub password: Option<String>,
    pub created_at: DateTime<Local>,
    pub updated_at: DateTime<Local>,
}

#[derive(Debug, Clone, Default)]
pub struct CreateUser {
    pub phone: String,
    pub email: String,
    pub password_salt: String,
    pub password: String,
}
