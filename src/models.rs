use chrono::{DateTime, Local};
pub struct App<ID> {
    pub id: ID,
    pub name: String,
    pub secret: String,
    pub secret_salt: String,
    pub created_at: DateTime<Local>,
    pub updated_at: DateTime<Local>,
}

pub struct CreateApp {
    pub name: String,
    pub secret: String,
    pub secret_salt: String,
}

pub struct QueryApp<ID>
where
    ID: Default,
{
    pub id_eq: Option<ID>,
}

pub struct User<ID> {
    pub id: ID,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub password_salt: Option<String>,
    pub password: Option<String>,
    pub secret: String,
    pub secret_salt: String,
    pub app_id: ID,
    pub created_at: DateTime<Local>,
    pub updated_at: DateTime<Local>,
}

#[derive(Debug, Clone, Default)]
pub struct CreateUser<ID>
where
    ID: Default,
{
    pub phone: Option<String>,
    pub email: Option<String>,
    pub password_salt: Option<String>,
    pub password: Option<String>,
    pub secret: String,
    pub secret_salt: String,
    pub app_id: ID,
}

#[derive(Debug, Clone, Default)]
pub struct QueryUser<ID>
where
    ID: Clone + Default,
{
    pub id_eq: Option<ID>,
    pub phone_eq: Option<String>,
    pub email_eq: Option<String>,
    pub app_id_eq: Option<ID>,
}

#[derive(Debug, Clone, Default)]
pub struct UpdateUser {
    pub secret: Option<String>,
    pub secret_salt: Option<String>,
}
