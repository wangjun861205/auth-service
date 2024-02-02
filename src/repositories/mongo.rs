use chrono::Utc;
use mongodb::{
    bson::{doc, Document},
    options::FindOneOptions,
    Database,
};

use crate::core::{
    entities::{CreateUser, User},
    error::Error,
    repository::Repository,
};

#[derive(Debug, Clone)]
pub struct MongodbRepository {
    db: Database,
}

impl MongodbRepository {
    pub fn new(db: Database) -> Self {
        Self { db }
    }
}

impl Repository for MongodbRepository {
    async fn exists_user(&self, phone: &str) -> Result<bool, Error> {
        Ok(self
            .db
            .collection::<Document>("users")
            .count_documents(doc! {"phone": phone}, None)
            .await
            .map_err(|e| Error::FailedToCheckExists(Box::new(e)))?
            == 1)
    }

    async fn insert_user(&self, user: &CreateUser) -> Result<String, Error> {
        let res = self
            .db
            .collection::<Document>("users")
            .insert_one(
                doc! {
                    "phone": &user.identifier,
                    "password": &user.password,
                    "password_salt": &user.password_salt,
                    "created_at": Utc::now().to_rfc3339(),
                    "updated_at": Utc::now().to_rfc3339(),
                },
                None,
            )
            .await
            .map_err(|e| Error::FailedToInsertUser(Box::new(e)))?;
        Ok(res
            .inserted_id
            .as_object_id()
            .ok_or(Error::FailedToInsertUser(Box::new(
                "未找到object id".to_owned(),
            )))?
            .to_hex())
    }

    async fn get_password_salt(&self, identifier: &str) -> Result<Option<String>, Error> {
        unimplemented!()
    }

    async fn get_id_by_credential(
        &self,
        identifier: &str,
        password: &str,
    ) -> Result<Option<String>, Error> {
        unimplemented!()
    }

    async fn set_key(&self, identifier: &str, key: &str) -> Result<(), Error> {
        unimplemented!()
    }

    async fn get_id_by_key(&self, token: &str) -> Result<Option<String>, Error> {
        unimplemented!()
    }
}
