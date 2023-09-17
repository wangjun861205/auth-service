use chrono::{DateTime, Local};

pub struct User<ID> {
    pub id: ID,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub password_salt: Option<String>,
    pub password: Option<String>,
    pub secret: String,
    pub secret_salt: String,
    pub created_at: DateTime<Local>,
    pub updated_at: DateTime<Local>,
}

#[derive(Debug, Clone, Default)]
pub struct CreateUser {
    pub phone: Option<String>,
    pub email: Option<String>,
    pub password_salt: Option<String>,
    pub password: Option<String>,
    pub secret: String,
    pub secret_salt: String,
}

#[derive(Debug, Clone, Default)]
pub struct QueryUser<ID>
where
    ID: Clone + Default,
{
    pub id_eq: Option<ID>,
    pub phone_eq: Option<String>,
    pub email_eq: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct UpdateUser {
    pub secret: Option<String>,
    pub secret_salt: Option<String>,
}
