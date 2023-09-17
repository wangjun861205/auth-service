use crate::core::entities::{CreateUser, QueryUser, UpdateUser, User};
use std::{error::Error as StdErr, fmt::Display};
pub trait Repository<ID>
where
    ID: Default + Clone + Display,
{
    async fn insert_user(&self, user: CreateUser) -> Result<ID, Box<dyn StdErr>>;
    async fn fetch_user(
        &mut self,
        query: QueryUser<ID>,
    ) -> Result<Option<User<ID>>, Box<dyn StdErr>>;
    async fn update_user(
        &self,
        query: QueryUser<ID>,
        user: UpdateUser,
    ) -> Result<i64, Box<dyn StdErr>>;
}
